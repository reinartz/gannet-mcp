%define crate_name gannet-mcp

Name:           gannet-mcp
Version:        0.2.0
Release:        1%{?dist}
Summary:        MCP server for web searching and webpages fetching

License:        MIT and Apache-2.0
URL:            https://github.com/reinartz/gannet-mcp
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

%changelog
* Mon Sep 14 2026  Ole Reinartz - 0.2.0-1
- service and mcp-config CLI subcommands, cargo-dist release pipeline
* Fri Jul 31 2026  Ole Reinartz - 0.1.0-1
- Initial RPM release
