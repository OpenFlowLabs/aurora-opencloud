# Installation images

## Image Types
- IPS

## Building Images
Images can be defined in the kdl language. The following commands are available as defenition commands.

### Imagefile
| Command       | Type   | Effect                                                                       | Notes                                                                                                                                                                                                                                                                             |
| ------------- | ------ | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| author        | string | Defines the Author of the Image                                              | Allowed formats are "John Doe" or "John Doe <john.doe@example.org>"                                                                                                                                                                                                               |
| name          | string | define a name for this image                                                 |                                                                                                                                                                                                                                                                                   |
| version       | string | defines the version of the format this definition was written egainst        | leaving this empty assumes 1                                                                                                                                                                                                                                                      |
| volume        | node   | Create a seperat Volume inside the image                                     | mount options can be specified as porperties and depend on the kernel used to setup the volume. Dataset type images must be ZFS based but other builders can use other properties. It is the implementations responsibility to inform users if a properties are supported or not. |
| remove        | string | Remove files and directories in the image                                    |                                                                                                                                                                                                                                                                                   |
| extract-tar   | string | Extract a tarfile into the image                                             | Unpacks the tarball in the `/` of the image                                                                                                                                                                                                                                       |
| onu           | node   | Install onu packages into the image                                          | Only works on IPS images that are illumos based                                                                                                                                                                                                                                   |
| devfsadm      | -      | run devfsadm inside the image to create a illumos `/dev` directory structure |                                                                                                                                                                                                                                                                                   |
| assemble-file | node   | Assembles a file from a directory of snippets                                |                                                                                                                                                                                                                                                                                   |
| group         | string | ensure a group with given name is present in the image                       | if the group exists this action is a noop                                                                                                                                                                                                                                         |
| user          | string | ensure a user with given name exists inside the image                        | if the user exists this action is a noop                                                                                                                                                                                                                                          |
| symlink       | node   | ensure a defined symlink exists                                              |                                                                                                                                                                                                                                                                                   |
| perms         | node   | ensure a given path has the defined permissions                              |                                                                                                                                                                                                                                                                                   |
| dir           | node   | ensure a given directory exists                                              |                                                                                                                                                                                                                                                                                   |
| file          | node   | ensure a given file with the defined content exists                          |                                                                                                                                                                                                                                                                                   |
| ips           | node   | setup the ips properties of the image                                        |                                                                                                                                                                                                                                                                                   |
| seed-smf      | -      | seed the smf repository inside the image                                     |                                                                                                                                                                                                                                                                                   |
| base-on       | string | base this image ontop of the image specidied here                            | use FMRI to address the image uniquely                                                                                                                                                                                                                                            |

```kdl
base-on img://openindiana.org/hipster
author John Doe <john.doe@example.com>
/*
The Publisher can not be mentioned here in the image fmri
the publisher part comes from DNS and the namespace the user can publish to
*/
name my-image
volume ...
assemble-file ...
ips ...

...
```
### volume
While only two properties are mentioned here, all ZFS properties are supported in dataset type images.
| Property   | type   | Effect                          | Notes                                                                                                                                                                   |
| ---------- | ------ | ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| name       | string | define the datasets name        | can contain /                                                                                                                                                           |
| mountpoint | string | set the mountpoint of the image | Optional if not set the volume is mounted under `/$NAME` where name is replaced with the volumes name folders that don't exist in the hierarchy are recursively created |

The properties of the volume can be type annotated for a specific driver. If that annotation is set all other implementations must ignore the property. If no type is annotated the property is valid for all volume drivers this image may be built by. Normally this is used only for ZFS performance tuning for things like PostgreSQL.
```kdl
volume data {
    mountpoint /var/lib/pgdata
    (zfs)checksum off
    (zfs)compression lz4
    copies 3
}
```
### assemble-file
| Property | type   | Effect                               |
| -------- | ------ | ------------------------------------ |
| dir      | string | directory where to find the snippets |
| output   | string | filename of the output file          |
| prefix   | string | only grab files that match prefix    |
| owner    | string | posix owner property                 |
| group    | string | posix group property                 |
| mode     | string | posix mode property                  |
```kdl
assemble-file /etc/system {
    dir system.d
    prefix assemble_
    owner root
    group bin
    mode 0644
}
```
### symlink
| Property | type   | Effect                     |
| -------- | ------ | -------------------------- |
| link     | string | name of the link           |
| target   | string | target of the link         |
| owner    | string | owner property of the link |
| group    | string | group property of the link |
```kdl
symlink /etc/mylink /etc/myorig {
    owner root
    group bin
}
```
### perm
| Property | type   | Effect               |
| -------- | ------ | -------------------- |
| path     | string | path to modify       |
| owner    | string | posix owner property |
| group    | string | posix group property |
| mode     | string | posix mode property  |
```kdl
perm /etc/daemon.d {
    owner root
    group bin
    mode 0644
}
```
### dir
| Property | type   | Effect               |
| -------- | ------ | -------------------- |
| path     | string | path to modify       |
| owner    | string | posix owner property |
| group    | string | posix group property |
| mode     | string | posix mode property  |
```kdl
dir /etc/daemon.d {
    owner root
    group bin
    mode 0644
}
```
### file 
| Property | type   | Effect                                        | Notes                                    |
| -------- | ------ | --------------------------------------------- | ---------------------------------------- |
| src      | string | source for the content of the file            |                                          |
| content  | string | content of the file as string                 |                                          |
| template | string | treats source or content as a template string | the format is jinja2 via the tera engine |
| path     | string | path of the file                              |                                          |
| owner    | string | posix owner property                          |                                          |
| group    | string | posix group property                          |                                          |
| mode     | string | posix mode property                           |                                          |
```kdl
file /etc/daemon.conf {
    is-template
    content "hello world!"
    src path/in/bundle.tmpl
    owner root
    group bin
    mode 0644
}
```

### ips
| Property          | type         | Effect                                                               | Notes |
| ----------------- | ------------ | -------------------------------------------------------------------- | ----- |
| packages          | Vec (string) | list of packages to install in the image                             |       |
| install-optionals | bool         | include the optional dependencies in the list of packages to install |       |
| property          | node         | ips properties of the image                                          |       |
| publisher         | node         | ips publishers to add                                                |       |
| ca                | node         | ips publisher CA to accept                                           |       |
| uninstall         | Vec (string) | list of packages to ensure are uninstalled                           |       |
| variant           | node         | change variant to value                                              |       |
| facet             | node         | change facet to value                                                |       |
| mediator          | node         | set a given mediator to an implementation or version                 |       |
| purge-history     | bool         | purge the history after operations                                   |       |
```kdl
ips {
    packages developer/gcc-11 golang golang-118
    uninstall userland-incorportation
    install-optionals
    property image.prop=false
    publisher openindiana.org https://pkg.openindiana.org/hipster
    ca openindiana.org /path/to/cert/in/image/bundle
    variant opensolaris.zone=global
    facet my.facet.name=true
    mediator mysql implementation=mariadb
    purge-history
}
```
### ips/property       
```kdl
ips {
    property image.prop=false
}
```
### ips/publisher
| Property  | type   | Effect |
| --------- | ------ | ------ |
| publisher | string |        |
| uri       | uri    |        |
```kdl
ips {
    publisher openindiana.org https://pkg.openindiana.org/hipster
}
```

### ips/ca
| Property  | type   | Effect                                        | Notes |
| --------- | ------ | --------------------------------------------- | ----- |
| publisher | string | publisher name for which the CA file is valid |       |
| certfile  | string | file of the CA                                |       |
```kdl
ips {
    ca openindiana.org /path/to/cert/in/image.bundle
}
```

### ips/variant
```kdl
ips {
    variant opensolaris.zone=global
}
```
### ips/facet
```kdl
ips {
    facet my.facet.name=true
}
```
### ips/mediator
| Property     | type   | Effect                                    | Notes |
| ------------ | ------ | ----------------------------------------- | ----- |
| name         | string | name of the mediator                      |       |
| implemtation | string | set this mediator to implementation given |       |
| version      | string | set this mediator to version given        |       |
```kdl
ips {
    mediator mysql implementation=mariadb
}
```

## Publishing Images
When publishing images to a image registry the namespace and hostname the image gets published to builds the first parts of the FMRI. The images name property builds the last part of the FMRI.

## Importing Images to the Host
To import an image into the system it is enough to specify the FMRI
```bash
imgadm import img://images.openindiana.org/hipster
```

## Examples

```kdl


