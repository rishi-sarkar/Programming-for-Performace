# Package management sample code

This code implements some package management functions (like Debian's apt) in Rust. I wrote it as assignment code for
ECE 459: Programming for Performance, to give students experience with writing nontrivial code in Rust.

You can load sets of available and installed packages and then perform dependency queries on these sets of packages.

## Initialization

Load the provided available and installed packages:

```
    $ load-defaults
    Packages available: 63846
    Packages installed: 3775
```

You can also load a specific packages or installed file with the `load-packages` or `load-installed` commands.

You can use short forms `ld`, `lp`, and `li`.

You can also load a CSV file with `load-csv`, to allow the verify part to be done independently of the other parts.

Part of your task will be to implement the available-packages and installed-packages parsers.

## Local state queries

* The `info` command prints out everything that is known about a package, integrating available and installed information.

```
    $ info bash
    Package: bash
    Version: 5.1-6
    MD5Sum: 95339277cdb215ea91851e02e8921e82
    Depends: base-files (>= 2.1.12), debianutils (>= 2.15)
    Installed-Version: 5.1-5+b1
    Newer-Available: true
```

`Newer-available` is present and `true` if the `Version` is newer than the `Installed-Version` according to
(hopefully) the Debian version comparison algorithm, or at least my implementation of it.

* The `deps` command prints the dependencies of a package; it's a formatted dump of information from the database.

```
    $ deps apt
    "apt" depends on "adduser, gpgv | gpgv2 | gpgv1, libapt-pkg6.0 (>= 2.3.14), debian-archive-keyring, libc6 (>= 2.33), libgcc-s1 (>= 3.0), libgnutls30 (>= 3.7.0), libseccomp2 (>= 2.4.2), libstdc++6 (>= 11), libsystemd0"
```

The `deps-available` command does a simple calculation: it prints information about whether all of the dependencies of a package are currently installed or not. Specifically, it iterates on the list of dependencies; for each dependency, it checks whether some package satisfying the dependency is installed. A dependency may be a disjunction A | B | C, and in that case, it checks whether one of A, B, or C is installed. Each dependency may be versioned (either exactly, with =, or with a constraint such as >=), and it checks whether the installed package has the right version.

```
    $ deps-available 3depict
    Package 3depict:
    - dependency "libc6 (>= 2.33)"
    + libc6 satisfied by installed version 2.33-1
    - dependency "libftgl2 (>= 2.4.0)"
    -> not satisfied
    - dependency "libgcc-s1 (>= 3.0)"
    + libgcc-s1 satisfied by installed version 11.2.0-13
    - dependency "libgl1"
    + libgl1 satisfied by installed version 1.4.0-1
    - dependency "libglu1-mesa | libglu1"
    + libglu1-mesa satisfied by installed version 9.0.1-1
    - dependency "libgomp1 (>= 6)"
    + libgomp1 satisfied by installed version 11.2.0-13
    - dependency "libgsl27 (>= 2.7.1)"
    -> not satisfied
    - dependency "libmgl7.6.0 (>= 2.5)"
    -> not satisfied
    - dependency "libpng16-16 (>= 1.6.2-1)"
    + libpng16-16 satisfied by installed version 1.6.37-3
    - dependency "libqhull8.0 (>= 2020.1)"
    + libqhull8.0 satisfied by installed version 2020.2-4
    - dependency "libstdc++6 (>= 11)"
    + libstdc++6 satisfied by installed version 11.2.0-13
    - dependency "libwxbase3.0-0v5 (>= 3.0.5.1+dfsg)"
    + libwxbase3.0-0v5 satisfied by installed version 3.0.5.1+dfsg-3
    - dependency "libwxgtk3.0-gtk3-0v5 (>= 3.0.5.1+dfsg)"
    + libwxgtk3.0-gtk3-0v5 satisfied by installed version 3.0.5.1+dfsg-3
    - dependency "libxml2 (>= 2.7.4)"
    + libxml2 satisfied by installed version 2.9.12+dfsg-5+b1
```

* The `transitive-dep-solution` command computes the unversioned transitive dependencies of a package: for each dependency d, it prints out d and all of d's dependencies, recursively. Where there is an alternative A | B | C, it chooses the first option A. This is a fairly simple work-list calculation.

```
    $ transitive-dep-solution 0ad
    "0ad" transitive dependency solution: "0ad-data, 0ad-data-common, libboost-filesystem1.74.0, libc6, libcurl3-gnutls, libenet7, libfmt8, libgcc-s1, libgl1, libgloox18, libicu67, libminiupnpc17, libopenal1, libpng16-16, libsdl2-2.0-0, libsodium23, libstdc++6, libvorbisfile3, libwxbase3.0-0v5, libwxgtk3.0-gtk3-0v5, libx11-6, libxml2, zlib1g, fonts-dejavu-core, fonts-freefont-ttf, fonts-texgyre, libbrotli1, libgnutls30, libgssapi-krb5-2, libidn2-0, libldap-2.4-2, libnettle8, libnghttp2-14, libpsl5, librtmp1, libssh2-1, libzstd1, gcc-11-base, libglvnd0, libglx0, libidn12, libopenal-data, libsndio7.0, libasound2, libdecor-0-0, libdrm2, libgbm1, libpulse0, libwayland-client0, libwayland-cursor0, libwayland-egl1, libxcursor1, libxext6, libxfixes3, libxi6, libxinerama1, libxkbcommon0, libxrandr2, libxss1, libxxf86vm1, libogg0, libvorbis0a, libexpat1, libcairo2, libgdk-pixbuf-2.0-0, libglib2.0-0, libgtk-3-0, libjpeg62-turbo, libnotify4, libpango-1.0-0, libpangocairo-1.0-0, libsm6, libtiff5, libxcb1, libx11-data, liblzma5, libgmp10, libhogweed6, libp11-kit0, libtasn1-6, libunistring2, libcom-err2, libk5crypto3, libkrb5-3, libkrb5support0, libsasl2-2, libssl1.1, libglx-mesa0, libbsd0, libasound2-data, libdrm-common, libwayland-server0, libasyncns0, libdbus-1-3, libsndfile1, libsystemd0, libwrap0, libx11-xcb1, libffi8, libxrender1, xkb-data, x11-common, libfontconfig1, libfreetype6, libpixman-1-0, libxcb-render0, libxcb-shm0, libgdk-pixbuf2.0-common, shared-mime-info, libmount1, libpcre3, libselinux1, adwaita-icon-theme, hicolor-icon-theme, libatk-bridge2.0-0, libatk1.0-0, libcairo-gobject2, libcolord2, libcups2, libepoxy0, libfribidi0, libharfbuzz0b, libpangoft2-1.0-0, libxcomposite1, libxdamage1, libgtk-3-common, fontconfig, libthai0, libice6, libuuid1, libdeflate0, libjbig0, libwebp6, libxau6, libxdmcp6, libkeyutils1, libsasl2-modules-db, debconf, libglapi-mesa, libxcb-dri2-0, libxcb-dri3-0, libxcb-glx0, libxcb-present0, libxcb-sync1, libxcb-xfixes0, libxshmfence1, libgl1-mesa-dri, libmd0, libflac8, libopus0, libvorbisenc2, libnsl2, lsb-base, fontconfig-config, libblkid1, libpcre2-8-0, gtk-update-icon-cache, libatspi2.0-0, libatk1.0-data, liblcms2-2, libudev1, libavahi-client3, libavahi-common3, libgraphite2-3, dconf-gsettings-backend, libthai-data, libdatrie1, libdb5.3, libdrm-amdgpu1, libdrm-intel1, libdrm-nouveau2, libdrm-radeon1, libelf1, libllvm12, libsensors5, libvulkan1, libtirpc3, ucf, libavahi-common-data, dconf-service, libdconf1, libpciaccess0, libedit2, libtinfo6, libz3-4, libsensors-config, libtirpc-common, coreutils, sensible-utils, default-dbus-session-bus"
```

* The `how-to-install` command is like `transitive-dep-solution` but filters out anything that is already installed and satisfied. Note that if there is an alternative, then it considers that dependency satisfied if any of the alternatives is installed and satisfied, and doesn't print it.

```
    $ how-to-install 3depict
    Package 3depict:
    "3depict" to install: "libftgl2, libgsl27, libmgl7.6.0, libgslcblas0, libhdf4-0, libhpdf-2.3.0, libmgl-data"
```

When a dependency is unsatisfied, there are two cases. (1) One of the alternatives is installed, but at the wrong version. In this case, compare apples and oranges, and pick the package with the highest available version number among the installed alternatives (hoping that it satisfies the dependency). (2) None of the alternatives is installed. Then pick the package with the highest version number among all available alternatives.

## Interaction with servers

The `enq-verify` command enqueues a request to a server for an md5sum for a (package, version) tuple. It optionally takes a version number to request from the server. In the absence of a version number, it requests the MD5sum for the available version.

```
    $ enq-verify bash
    queueing request http://ece459.patricklam.ca:4590/rest/v1/checksums/bash/5.1-6
    $ enq-verify libc6 28
    queueing request http://ece459.patricklam.ca:4590/rest/v1/checksums/libc6/28
```

The `execute` (and `quit`) commands execute all enqueued requests using nonblocking I/O, wait for the responses, and compare local MD5 to returned MD5.
```
    $ quit
    verifying bash, matches: true
    got error 404 on request for package libc6 version 28
```
Of course, the `quit` command also quits.

If a student solution blocks, then we'd expect to see a much longer 
expected queue draining time.

## Internal instrumentation

We used two of the commands in development; they aren't intended for student use.

`output-md5s` will create a csv file containing all MD5s of available
packages, in a form that the package-verifier can understand.

`test-version-compare` provides an interactive test interface for the
somewhat hairy Debian version comparison algorithm. Specify two
versions. It'll parse them and tell you the relation between the first
version and the second one. There should be unit tests that encode a few
of these.

## Bonus: Command completion

It would be really cool if someone implemented history completion
(see the line `let mut rl = Editor::<()>::new()`; there's a hook for 
custom completion code there).