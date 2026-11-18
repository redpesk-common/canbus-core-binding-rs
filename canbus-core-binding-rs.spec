%global debug_package %{nil}

%bcond_with coverage

Name: canbus-core-binding-rs
Version: 0.0.0
Release: 1%{?dist}
Summary: Rust CAN bindings

License: GPL-3.0
URL: https://github.com/redpesk-common/canbus-core-binding-rs
Source0: %{name}-%{version}.tar.gz
Source1: vendor.tar.bz2
Source2: cargo_config

BuildRequires:   rust >= 1.70
BuildRequires:   cargo >= 1.70
BuildRequires:   clang-devel
BuildRequires:  json-c-devel
BuildRequires:  pkgconfig(json-c)
BuildRequires:  clang
BuildRequires:  pkgconfig(afb-binding)
BuildRequires:  iputils
BuildRequires: glibc-devel
%description
Rust bindings and helper libraries to bridge Linux SocketCAN / BCM with the AFB v4 application framework.

%package redtest
Summary: redtest package
Requires: %{name} = %{version}-%{release}
Requires: afb-test-py
Requires: afb-libpython
Requires: iproute
Requires: can-utils
Requires: kernel-modules-extra
Requires: git
%if %{with coverage}
Requires: llvm
Requires: python3-pip
%endif

%description redtest
This package contains the redtest test suite. Coverage support is disabled by
default and can be enabled at build time with --with coverage.

%prep
%autosetup -p 1

# Bring vendored deps + Cargo.lock into the build tree
tar -xjf %{SOURCE1}

mkdir -p .cargo
cp %{SOURCE2} .cargo/config

%build
%if %{with coverage}
export RUSTFLAGS="-C instrument-coverage -C debuginfo=2 -C link-dead-code"
%endif

cargo build --offline --release --target %{_arch}-unknown-linux-gnu

%if %{with coverage}
cargo test --release --all-targets --all-features
%endif

%install
mkdir -p %{buildroot}%{_prefix}/redpesk/%{name}/lib
cp ./target/%{_arch}-unknown-linux-gnu/release/libafb_sockcan.so %{buildroot}%{_prefix}/redpesk/%{name}/lib

mkdir -p %{buildroot}%{_prefix}/redpesk/%{name}/.rpconfig

# Install redtest scripts (for testing)
mkdir -p %{buildroot}%{_libexecdir}/redtest/%{name}/functional/
cp redtest/run-redtest %{buildroot}%{_libexecdir}/redtest/%{name}/
cp tests/*.sh          %{buildroot}%{_libexecdir}/redtest/%{name}/
cp tests/functional/*.py          %{buildroot}%{_libexecdir}/redtest/%{name}/functional/

# Install redtest binding (for testing)
mkdir -p %{buildroot}%{_libexecdir}/redtest/%{name}/lib/
cp ./target/%{_arch}-unknown-linux-gnu/release/libafb_can_multiplexing.so %{buildroot}%{_libexecdir}/redtest/%{name}/lib/
cp ./target/%{_arch}-unknown-linux-gnu/release/libafb_model3.so           %{buildroot}%{_libexecdir}/redtest/%{name}/lib/
cp ./target/%{_arch}-unknown-linux-gnu/release/libafb_bms.so              %{buildroot}%{_libexecdir}/redtest/%{name}/lib/

mkdir -p %{buildroot}%{_libexecdir}/redtest/examples/samples/dbc_multiplexing/dbc/
mkdir -p %{buildroot}%{_libexecdir}/redtest/examples/samples/model3/binding-config/
mkdir -p %{buildroot}%{_libexecdir}/redtest/examples/samples/model3/dbc/
mkdir -p %{buildroot}%{_libexecdir}/redtest/examples/samples/bms/binding-config/
mkdir -p %{buildroot}%{_libexecdir}/redtest/examples/samples/bms/dbc/

cp ./examples/samples/dbc_multiplexing/dbc/multiplexing.dbc     %{buildroot}%{_libexecdir}/redtest/examples/samples/dbc_multiplexing/dbc/
cp ./examples/samples/model3/binding-config/binding-config.json %{buildroot}%{_libexecdir}/redtest/examples/samples/model3/binding-config/
cp ./examples/samples/model3/dbc/model3can.dbc                  %{buildroot}%{_libexecdir}/redtest/examples/samples/model3/dbc/
cp ./examples/samples/bms/binding-config/binding-config.json    %{buildroot}%{_libexecdir}/redtest/examples/samples/bms/binding-config/
cp ./examples/samples/bms/dbc/BMS.dbc                           %{buildroot}%{_libexecdir}/redtest/examples/samples/bms/dbc/

cp ./tests/requirements.txt %{buildroot}%{_libexecdir}/redtest/%{name}/

%if %{with coverage}
# Enable LLVM coverage in the installed redtest runner.
sed -i 's/^COVERAGE=no$/COVERAGE=yes/' \
  %{buildroot}%{_libexecdir}/redtest/%{name}/run-redtest
%endif

%files
%dir %{_prefix}/redpesk/%{name}
%dir %{_prefix}/redpesk/%{name}/.rpconfig
#%%{_prefix}/redpesk/%%{name}/.rpconfig/*
%dir %{_prefix}/redpesk/%{name}/lib
%{_prefix}/redpesk/%{name}/lib/libafb_sockcan.so

%files redtest
%defattr(-,root,root)
%dir %{_libexecdir}/redtest/%{name}/
%dir %{_libexecdir}/redtest/%{name}/lib/
%{_libexecdir}/redtest/%{name}/lib/*.so
%dir %{_libexecdir}/redtest/%{name}/
%{_libexecdir}/redtest/%{name}/*.sh
%{_libexecdir}/redtest/%{name}/run-redtest
%dir %{_libexecdir}/redtest/%{name}/functional/
%{_libexecdir}/redtest/%{name}/functional/*.py

%dir %{_libexecdir}/redtest/examples/samples/dbc_multiplexing/dbc/
%dir %{_libexecdir}/redtest/examples/samples/model3/binding-config/
%dir %{_libexecdir}/redtest/examples/samples/model3/dbc/
%dir %{_libexecdir}/redtest/examples/samples/bms/binding-config/
%dir %{_libexecdir}/redtest/examples/samples/bms/dbc/

%{_libexecdir}/redtest/examples/samples/dbc_multiplexing/dbc/multiplexing.dbc
%{_libexecdir}/redtest/examples/samples/model3/binding-config/binding-config.json
%{_libexecdir}/redtest/examples/samples/model3/dbc/model3can.dbc
%{_libexecdir}/redtest/examples/samples/bms/binding-config/binding-config.json
%{_libexecdir}/redtest/examples/samples/bms/dbc/BMS.dbc

%{_libexecdir}/redtest/%{name}/requirements.txt

%changelog
