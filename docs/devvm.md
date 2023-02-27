Concept of a tool like vagrant to make development based on recipies easier but enhanced with the capability to share parts of the recipe

## Definition files
```kdl
project "solarm-gate" {
	// Can be omited if the definition is located inside the git repository
	git "git@server.com:path/to/repo.git"

	// If instead of pulling the repo from the reomte branch it is desired to 
	// share the repo from a 9p share one can use this directive think vagrant
	// share
	// to share another folder other than . pass a relative path as argument
	// share "sub/folder"
}

specs {
	cpus 2
	// Either a fixed assignment can be made like this
	memory "4G"
	// Or a dynamic assignment that takes the host memory as basis
	// the argument becomes the minimal memory assigned to the vm
	// fractions are based on host memory TODO: define supported fractional steps
	// memory "4G" max="1/4"
}

// The ISA of the VM image determines the emulator launched
// Emit a warning 
image "img://solarm.org/hvm/braich/rolling"

config {
	// install packages
	// include other config nodes from file
	// run scripts on boot and provisioning
}
```