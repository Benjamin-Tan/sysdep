# Sysdep

A simple system dependency tool to list/install the apt/pip dependencies based on a toml file, written in Rust. The goal of this tool is to allow developer to specify the system dependencies of any project that has software packages written in various programming languages (C, C++, Python). The dummy example .toml can be found in [examples](./examples/).

## Highlights

- Utilise only the **official** package manager of Debian and Python
- Support for both **aarch64** and **x86_64**.
- Option to specify **large** dependencies that take time to install, which is useful for caching.
- Suitable for [monorepo and multirepo](https://medium.com/@magenta2127/monorepo-vs-multi-repo-vs-monolith-7c4a5f476009).

## Install

```bash
# To install to `~/local/bin`, ensure directory exists and `PATH=~/.local/bin:$PATH`
# For latest version
curl -L https://github.com/Benjamin-Tan/sysdep/releases/latest/download/sysdep-$(arch)-unknown-linux-gnu.tar.gz | tar -xz -C ~/.local/bin

# For specific version,
curl -L https://github.com/Benjamin-Tan/sysdep/releases/download/v0.2.0/sysdep-$(arch)-unknown-linux-gnu.tar.gz | tar -xz -C ~/.local/bin

# To install to `/usr/local/bin`
curl -L https://github.com/Benjamin-Tan/sysdep/releases/download/v0.2.0/sysdep-$(arch)-unknown-linux-gnu.tar.gz | sudo tar -xz -C /usr/local/bin
```

## Usage

```bash
# List the dependencies of the current directory
sysdep list

# List the dependencies in all subdirectories recursively '**'
sysdep list '**'

# List the dependencies in all subdirectories recursively, with matching name 'example1' or 'example2' only.
sysdep list '**/{example1,example2}'

# Install the dependencies in all subdirectories recursively
sysdep install '**'

# Only list the pip dependencies
sysdep list '**' pip

# Only list the apt dependencies
sysdep list '**' apt

# Only list the large apt dependencies
sysdep list '**' apt --large-dep

# For more options
sysdep
sysdep --help
```

## Why use sysdep?

Below are the existing tools to install the system dependencies. But none of them has native support to install apt dependencies, except for `rosdep`.

### ROS

- [rosdep](https://github.com/ros-infrastructure/rosdep)
  - require `package.xml` and the updated `rosdep` rules
  - raise a PR to contribute to the public `rosdep` rules
  - edit the file directly for private `rosdep` rules
  - `package.xml` could represent both the source and system dependencies, apt or pip dependencies
  - to find out the actual dependencies being installed, need to resolve and lookup the `rosdep` rules
  - unable to install the [specific version](https://robotics.stackexchange.com/questions/98835/rosdep-install-specific-version-of-dependencies)

### C/C++

- [conan](https://conan.io/): a modern tool mainly targetting at C/C++ package only, require to create and host the Conan package if it is not available on [ConanCenter](https://conan.io/center).

### Python

Numerous ways to specify the Python dependencies only.

- [requirements.txt](https://pip.pypa.io/en/stable/reference/requirements-file-format/)
- [setup.py](https://packaging.python.org/en/latest/guides/distributing-packages-using-setuptools/#install-requires)
- [pyproject.toml](https://packaging.python.org/en/latest/guides/distributing-packages-using-setuptools/#install-requires)
- [rye](https://github.com/astral-sh/rye)
- [poetry](https://github.com/python-poetry/poetry)

### Cross-platform, language-agnostic binary package manager

Conda ecosystem, require to build and upload the conda package.

- [conda](https://github.com/conda/conda)
- [pixi](https://github.com/prefix-dev/pixi)
