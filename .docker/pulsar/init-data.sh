#!/bin/bash

out=`bin/pulsar-admin tenants create timada 2>&1`

if [ ! -z "$out" ] && [[ ! "$out" =~ .*"Tenant already exist".* ]]; then
    exit 1
fi

out=`bin/pulsar-admin namespaces create timada/pikav 2>&1`

if [ ! -z "$out" ] && [[ ! "$out" =~ .*"Namespace already exists".* ]]; then
    exit 1
fi
