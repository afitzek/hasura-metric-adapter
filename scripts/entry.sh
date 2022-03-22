#!/bin/bash

APP=${APP:=/metrics}

if [[ -z "${LOG_FILE}" ]]; then
    echo "NEED TO SET LOG_FILE ENVVAR!!!"
    exit -1;
fi

if [[ ! -r "${LOG_FILE}" ]]; then
    echo "Log file ${LOG_FILE} doesn't exists!"
    exit -1;
fi

tail -n+1 -f ${LOG_FILE} | $APP