use std::cmp::Ordering;

use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveTime};

use crate::{db::InitUuid, model::group::GroupUuid, uuid_impls};

use super::properties::OriginType;

pub(crate) type ModelDatetime = Datetime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Datetime {
    pub uuid: DatetimeUuid,
    pub origin_type: OriginType,
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub timezone: i32,
    pub group_uuid: GroupUuid,
    pub datetime: DateTime<Local>,
}

uuid_impls!(DatetimeUuid);

impl Datetime {
    pub fn init(
        date: NaiveDate,
        time: Option<NaiveTime>,
        timezone: i32,
        group_uuid: GroupUuid,
    ) -> Self {
        Self {
            uuid: DatetimeUuid::init(),
            origin_type: OriginType::CsvImport,
            date,
            time,
            timezone,
            group_uuid,
            datetime: Self::compute_datetime(date, time, timezone),
        }
    }

    pub fn init_datetime(
        datetime: DateTime<Local>,
        group_uuid: GroupUuid,
    ) -> Self {
        Self::init(
            datetime.date_naive(),
            Some(datetime.naive_local().time()),
            datetime.offset().local_minus_utc(),
            group_uuid,
        )
    }

    pub fn new(
        uuid: DatetimeUuid,
        origin_type: OriginType,
        date: NaiveDate,
        time: Option<NaiveTime>,
        timezone: i32,
        group_uuid: GroupUuid,
    ) -> Self {
        let datetime = Self::compute_datetime(date, time, timezone);
        Self {
            uuid,
            origin_type,
            date,
            time,
            timezone,
            group_uuid,
            datetime,
        }
    }

    fn compute_datetime(
        date: NaiveDate,
        time: Option<NaiveTime>,
        timezone: i32,
    ) -> DateTime<Local> {
        date.and_time(time.unwrap_or_default())
            .and_local_timezone(FixedOffset::east_opt(timezone).unwrap())
            .unwrap()
            .with_timezone(&Local)
    }

    pub fn cmp_datetime(&self, other: &Self) -> Ordering {
        self.datetime.cmp(&other.datetime)
    }
}
