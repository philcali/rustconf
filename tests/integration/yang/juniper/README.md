# Juniper YANG Models (local fallback)

This directory is a **fallback** for when the `juniper-yang` git submodule is
not initialised. The preferred approach is to use the submodule:

```bash
git submodule update --init tests/integration/yang/juniper-yang
```

The `build.rs` script checks the submodule first. If it is present, models
are read from `juniper-yang/<release>/<revision>/native/conf-and-rpcs/junos/conf/models/`
and this directory is ignored.

## Manual population (fallback only)

If you cannot use the submodule, copy the needed `.yang` files here from the
[Juniper/yang](https://github.com/Juniper/yang) repository:

```bash
git clone --depth 1 https://github.com/Juniper/yang.git /tmp/juniper-yang
cp /tmp/juniper-yang/23.4/23.4R1/native/conf-and-rpcs/junos/conf/models/*.yang \
   tests/integration/yang/juniper/
```

Also copy IETF dependency models into `../ietf/`:

```bash
cp /tmp/juniper-yang/23.4/23.4R1/ietf/models/*.yang tests/integration/yang/ietf/
```
