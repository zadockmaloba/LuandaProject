# Linux Dependencies

The Rust engine crate uses the following platform-specific dependencies on Linux:

```toml
vulkano = "0.35.2"
vulkano-shaders = "0.35.0"
```

`vulkano-shaders` compiles GLSL shaders to SPIR-V at build time using
[shaderc](https://github.com/google/shaderc). If a system-installed `libshaderc`
is not found, it will attempt to build shaderc from source, which requires
CMake, Ninja, and Python and may fail against newer versions of GCC (13+) due
to an outdated bundled glslang (missing `#include <cstdint>`).

## System packages

Install the following packages before building to avoid the from-source build:

### Ubuntu / Debian
```sh
sudo apt install libshaderc-dev libvulkan-dev vulkan-tools
```

### Arch Linux
```sh
sudo pacman -S shaderc vulkan-devel vulkan-tools
```

### Fedora
```sh
sudo dnf install libshaderc-devel vulkan-loader-devel vulkan-tools
```

| Package | Purpose |
|---|---|
| `libshaderc-dev` / `shaderc` | Provides `libshaderc` so `shaderc-sys` skips its from-source build |
| `libvulkan-dev` / `vulkan-devel` | Vulkan loader headers and library required by `vulkano` |
| `vulkan-tools` | Includes `vulkaninfo` for verifying driver/device availability at runtime |

## Build tools (only needed if building shaderc from source)

If the system shaderc package is unavailable, ensure these are on `PATH`:

- [CMake](https://cmake.org/) ≥ 3.10
- [Ninja](https://ninja-build.org/)
- Python 3

## Runtime requirements

A Vulkan-capable GPU with an up-to-date driver is required at runtime. On
machines without a GPU, software rasterization is available via
[lavapipe](https://docs.mesa3d.org/drivers/llvmpipe.html):

```sh
# Ubuntu / Debian
sudo apt install mesa-vulkan-drivers

# Arch
sudo pacman -S mesa

# Fedora
sudo dnf install mesa-vulkan-drivers
```
