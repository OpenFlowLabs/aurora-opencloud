#!/usr/bin/bash

set -x

DATASETS=$(zfs list -Ho name -r rpool/zones | grep -Ev 'root|vroot' | tail -n +2 | grep -v $(cat /etc/zimages/* | jq .uuid | sed 's/"//g'))

for ds in $DATASETS; do
	zfs destroy -r $ds
done