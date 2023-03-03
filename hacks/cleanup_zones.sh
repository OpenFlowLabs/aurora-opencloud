#!/usr/bin/bash

set -x

for zone in $(zoneadm list -cp | cut -d ":" -f 2 | grep -v global); do 
	sudo zoneadm -z $zone halt
	sudo zoneadm -z $zone uninstall -F
	sudo zonecfg -z $zone delete -F
done
