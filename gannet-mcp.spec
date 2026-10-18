%define crate_name gannet-mcp

Name:           gannet-mcp
Version:        0.2.2
Release:        1%{?dist}
Summary:        MCP server for web searching and webpages fetching

License:        MIT and Apache-2.0
URL:            https://github.com/reinartz/gannet-mcp
Icon:           gannet-mcp.png
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  rust-srpm-macros

%description
A Model Context Protocol (MCP) server that provides web search and webpage
fetching capabilities. Supports multiple search providers including DuckDuckGo,
Serper, SearXNG, and Bright Data.

%prep
%setup -q -n %{crate_name}-%{version}

%generate_buildrequires
%cargo_generate_buildrequires

%build
%cargo_build

%install
%cargo_install

# Install systemd unit
mkdir -p %{buildroot}%{_sysconfdir}/gannet-mcp
install -Dm 644 systemd/gannet-mcp.service %{buildroot}%{_unitdir}/gannet-mcp.service
install -Dm 644 systemd/gannet-mcp.conf %{buildroot}%{_sysconfdir}/gannet-mcp.conf
install -Dm 644 systemd/gannet-mcp.sysusers %{buildroot}%{_sysusersdir}/gannet-mcp.conf
install -Dm 644 systemd/gannet-mcp.tmpfiles %{buildroot}%{_tmpfilesdir}/gannet-mcp.conf

# Install desktop icons
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/16x16/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/32x32/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/48x48/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/64x64/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/128x128/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/256x256/apps
install -Dm 644 assets/icons/gannet-mcp-16.png %{buildroot}%{_datadir}/icons/hicolor/16x16/apps/gannet-mcp.png
install -Dm 644 assets/icons/gannet-mcp-32.png %{buildroot}%{_datadir}/icons/hicolor/32x32/apps/gannet-mcp.png
install -Dm 644 assets/icons/gannet-mcp-48.png %{buildroot}%{_datadir}/icons/hicolor/48x48/apps/gannet-mcp.png
install -Dm 644 assets/icons/gannet-mcp-64.png %{buildroot}%{_datadir}/icons/hicolor/64x64/apps/gannet-mcp.png
install -Dm 644 assets/icons/gannet-mcp-128.png %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/gannet-mcp.png
install -Dm 644 assets/icons/gannet-mcp-256.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/gannet-mcp.png

%check
%cargo_test

%files
%doc README.md
%license LICENSE-MIT LICENSE-APACHE
%{_bindir}/%{name}
%{_unitdir}/gannet-mcp.service
%{_sysconfdir}/gannet-mcp.conf
%{_sysusersdir}/gannet-mcp.conf
%{_tmpfilesdir}/gannet-mcp.conf
%{_datadir}/icons/hicolor/16x16/apps/gannet-mcp.png
%{_datadir}/icons/hicolor/32x32/apps/gannet-mcp.png
%{_datadir}/icons/hicolor/48x48/apps/gannet-mcp.png
%{_datadir}/icons/hicolor/64x64/apps/gannet-mcp.png
%{_datadir}/icons/hicolor/128x128/apps/gannet-mcp.png
%{_datadir}/icons/hicolor/256x256/apps/gannet-mcp.png

%changelog
* Sun Sep 27 2026  Ole Reinartz - 0.2.2-1
- Windows ARM64 (aarch64-pc-windows-msvc) build and MSI installer via native windows-11-arm runner
* Sun Sep 27 2026  Ole Reinartz - 0.2.1-1
- Windows MSI installer (unsigned) via cargo-dist, EnvironmentFile dash mismatch fix, mcp-config --stdio --http now selects HTTP like the server flags, Phase 6 docs (INSTALL/SERVICES/STDIO guides, README badges)
* Mon Sep 14 2026  Ole Reinartz - 0.2.0-1
- service and mcp-config CLI subcommands, cargo-dist release pipeline
* Fri Jul 31 2026  Ole Reinartz - 0.1.0-1
- Initial RPM release
