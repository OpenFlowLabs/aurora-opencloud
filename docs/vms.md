# Virtual Machines

## Types:
- Zones
    - IPS Linked (lipkg)
    - IPS Unlinked (nlipkg/ipkg)
    - image (joyent/alien/oci)
    - oci runtime spec
- Hypervisors
    - bHyve
    - Propolis

# Definition files
Basics:
```kdl
tenant "openflowlabs.com"

name "oibldofhipster"

image "img://openindiana.org/hvm/hipster/202304"

specs {
	cpus 4
	memory "16G"
}

// bHyve is the default
(bhyve)vm {
	(vioscsi)disk "vda" {
		size "50G"
	}
	(vioscsi)disk "vdb" size="50G"
}

// Network names are always bound to a tenant and are unique per tenant
(virtio)net "testnet" {
	// Override the name of the NIC (defaults to $NETNAME$N where $N is a 
	// free nic number)
	name "testnet0"
	// Multiple addresses per card can be defined
	// IPv6only is the default
	address "addrconf"
	address "2001:db8::5"

	// The default gateway is not required as one is noted in the networks definition so this is a second gateway the primary is commented out here
	// gateway "2001:db8::1"
	gateway "2001:db8::2"
}

// The Default is openindiana's sysconfig although a simplistic cloud-init is supported by setting the config type to (cloud-config) thus an empty node like this:
// (cloud-config) config
// uses the net node above to set the name inside the VM
// if the VM is a linux image cloud-config must be mentioned.
// I will not default to typing the VM based on the OS as this is not
// needed. Sysconfig could eventually be ported and otherwise I'll have
// to maintain a list of operating systems.... I don't want that
config {
	// Defaults to the VM's name node above
	// hostname "oibldofhipster"
	// TODO further runtime config functionality
}
```

Future Ideas:
- 