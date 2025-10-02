#!/bin/bash

if [ $1 = '-q' ]; then
    quiet='1'
fi

function print_pretty {
    if [ -z $quiet ]; then
        echo "-"
        echo ""
    fi

    echo ">>>>> BUILD SCRIPT: '$1' returned code: '$2'"

    if [ -z $quiet ]; then
        echo ""
        echo "-"
    fi
}

function exec_command {

    if [ -z $quiet ]; then
        echo ""
        echo "               Starting with ${1}"
        echo ""
        echo "######################################################"
    fi

    if [ $quiet ]; then
        eval "${2}" > /dev/null 2> /dev/null
        result=$?
        executed="$2"
    else
        eval "${3}"
        result=$?
        executed="$3"
    fi

    if [ $result -eq 0 ]; then
        total_result="${result}"
    else
        unset total_result
    fi

    if [ -z $quiet ]; then
        echo "######################################################"
        echo ""
        echo "               Finished with ${1}"
        echo ""
    fi

    print_pretty "${executed}" "${result}"
}

exec_command "Build" 'cargo build --release -q' 'cargo build --release -v'

exec_command "Tests" 'cargo test -q' 'cargo test -v'

if [ $total_result ]; then
    read -p "Do you want to copy the binary to '/Users/tomellm/Applications/bread_manager'? (y/n) " yn
    case $yn in 
        [yY] ) cp ./target/release/bread-manager /Users/tomellm/Applications/bread_manager ;;
        * ) echo "    didn't copy...";;
    esac

else
    echo ""
    echo "ERROR: one of the commands returned an error. Run without '-q' for more info."
    echo ""
fi

#cp ./target/release/bread-manager /Users/tomellm/Applications/bread_manager


