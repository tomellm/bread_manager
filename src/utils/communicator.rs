use futures::future::BoxFuture;
use lazy_async_promise::{ImmediateValuePromise, ImmediateValueState};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use tokio::sync::{
    mpsc,
    watch::{self, Ref},
};

use super::changer::{self, ActionType, Response};

pub struct DataContainer<Key, Value, Writer>
where
    Key: Clone + Send + Sync + 'static,
    Value: Clone + GetKey<Key> + Send + Sync + 'static,
    Writer: Storage<Key, Value>,
{
    data: HashMap<Key, Value>,
    change_reciver: changer::Reciver<Key, Value>,
    data_broadcast: watch::Sender<HashMap<Key, Value>>,
    writer: Writer,
    processing_responses: Vec<ImmediateValuePromise<Response<Key, Value>>>,
}

pub trait GetKey<Key> {
    fn get_key(&self) -> Key;
}

pub trait Storage<Key, Value>
where
    Self: Send + Sync,
    Key: Clone + Send + Sync + 'static,
    Value: GetKey<Key> + Clone + Send + Sync + 'static,
{
    fn handle_action(
        &mut self,
        action: changer::Action<Key, Value>,
    ) -> ImmediateValuePromise<Response<Key, Value>> {
        println!("got a action about to handle it");
        let (action, responder) = action.get();
        let future = match action {
            ActionType::Set(value) => self.set(value),
            ActionType::SetMany(values) => self.set_many(values),
            ActionType::Update(value) => self.update(value),
            ActionType::UpdateMany(values) => self.update_many(values),
            ActionType::Delete(key) => self.delete(key),
            ActionType::DeleteMany(values) => self.delete_many(values),
            ActionType::GetAll(_) => self.get_all(),
        };
        ImmediateValuePromise::new(async move {
            let response = future.await;
            let _ = responder.send(response.clone());
            Ok(response)
        })
    }
    fn set(&mut self, value: Value) -> BoxFuture<'static, Response<Key, Value>>;
    fn set_many(&mut self, values: Vec<Value>) -> BoxFuture<'static, Response<Key, Value>>;
    fn update(&mut self, value: Value) -> BoxFuture<'static, Response<Key, Value>>;
    fn update_many(&mut self, values: Vec<Value>) -> BoxFuture<'static, Response<Key, Value>>;
    fn delete(&mut self, key: Key) -> BoxFuture<'static, Response<Key, Value>>;
    fn delete_many(&mut self, keys: Vec<Key>) -> BoxFuture<'static, Response<Key, Value>>;
    fn get_all(&mut self) -> BoxFuture<'static, Response<Key, Value>>;
    fn setup(&mut self, drop: bool) -> BoxFuture<'static, Result<Vec<Value>, ()>>;
}

impl<Key, Value, Writer> DataContainer<Key, Value, Writer>
where
    Key: core::cmp::Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    Value: GetKey<Key> + Clone + Send + Sync + 'static,
    Writer: Storage<Key, Value>,
{
    pub fn new(mut writer: Writer, drop: bool) -> Self {
        let values = Self::setup_writer(&mut writer, drop);

        let data = values
            .into_iter()
            .map(|v| (v.get_key(), v))
            .collect::<HashMap<_, _>>();
        let (changer_sender, changer_reciver) = mpsc::channel::<changer::Action<Key, Value>>(5);
        let (data_sender, _) = watch::channel(data.clone());

        Self {
            data,
            writer,
            data_broadcast: data_sender,
            processing_responses: vec![],
            change_reciver: changer::Reciver::new(changer_sender, changer_reciver),
        }
    }

    fn setup_writer(writer: &mut Writer, drop: bool) -> Vec<Value> {
        let future = writer.setup(drop);
        let mut promise = ImmediateValuePromise::new(async move { Ok(future.await.unwrap()) });
        loop {
            match promise.poll_state() {
                ImmediateValueState::Success(vec) => return vec.clone(),
                ImmediateValueState::Error(_) => panic!("Setting up the db didnt work!"),
                _ => continue,
            }
        }
    }

    pub fn signal(&mut self) -> Communicator<Key, Value> {
        Communicator {
            viewer: self.viewer(),
            changer: self.changer(),
        }
    }

    pub fn changer(&self) -> changer::Sender<Key, Value> {
        self.change_reciver.get_sender()
    }

    pub fn viewer(&mut self) -> watch::Receiver<HashMap<Key, Value>> {
        self.data_broadcast.subscribe()
    }

    pub fn update(&mut self) {
        self.recive_new();
        self.consume_finished();
    }

    fn consume_finished(&mut self) {
        let loading_responses = self
            .processing_responses
            .drain(..)
            .collect::<Vec<_>>()
            .into_iter()
            .fold(vec![], |mut acc, mut res| {
                match res.poll_state() {
                    ImmediateValueState::Empty | ImmediateValueState::Error(_) => (),
                    ImmediateValueState::Updating => acc.push(res),
                    ImmediateValueState::Success(value) => self.update_data(value),
                }
                acc
            });
        self.processing_responses = loading_responses;
    }

    fn recive_new(&mut self) {
        if let Some(actions) = self.change_reciver.recive() {
            for action in actions {
                let action_future = self.writer.handle_action(action);
                self.processing_responses.push(action_future);
            }
        }
    }

    fn update_data(&mut self, response: &Response<Key, Value>) {
        if let Response::Worked(action) = response {
            self.data_broadcast.send_modify(|data| match action {
                changer::ActionType::Set(value) | changer::ActionType::Update(value) => {
                    println!("inserted stuff");
                    data.insert(value.get_key(), value.clone());
                }
                changer::ActionType::SetMany(values) | changer::ActionType::UpdateMany(values) => {
                    println!("inserted many");
                    data.extend(values.clone().into_iter().map(|v| (v.get_key(), v)));
                }
                changer::ActionType::Delete(key) => {
                    println!("deleted stuff");
                    data.remove(key);
                }
                changer::ActionType::DeleteMany(keys) => {
                    println!("deleted many");
                    data.retain(|k, _| !keys.contains(k));
                }
                changer::ActionType::GetAll(_) => (),
            });
        }
    }
}

#[derive(Clone)]
pub struct Communicator<Key, Value>
where
    Key: Clone + Send + Sync,
    Value: GetKey<Key> + Clone + Send + Sync,
{
    viewer: watch::Receiver<HashMap<Key, Value>>,
    changer: changer::Sender<Key, Value>,
}

impl<Key, Value> Communicator<Key, Value>
where
    Key: Clone + Send + Sync,
    Value: GetKey<Key> + Clone + Send + Sync,
{
    pub fn view(&self) -> Ref<'_, HashMap<Key, Value>> {
        self.viewer.borrow()
    }
    pub fn update(&mut self) {
        self.changer.update_sender();
    }
    pub fn viewer(&self) -> watch::Receiver<HashMap<Key, Value>> {
        self.viewer.clone()
    }
}

impl<Key, Value> Deref for Communicator<Key, Value>
where
    Key: Clone + Send + Sync,
    Value: GetKey<Key> + Clone + Send + Sync,
{
    type Target = changer::Sender<Key, Value>;
    fn deref(&self) -> &Self::Target {
        &self.changer
    }
}

impl<Key, Value> DerefMut for Communicator<Key, Value>
where
    Key: Clone + Send + Sync,
    Value: GetKey<Key> + Clone + Send + Sync,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.changer
    }
}
