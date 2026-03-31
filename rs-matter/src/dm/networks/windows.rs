/*
 *
 *    Copyright (c) 2020-2022 Project CHIP Authors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! A module containing NetifDiag and NetChangeNotif implementations for Windows systems.

use core::net::{Ipv4Addr, Ipv6Addr};

use alloc::string::String;
use alloc::vec::Vec;

use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};

use crate::dm::clusters::gen_diag::{InterfaceTypeEnum, NetifDiag, NetifInfo};
use crate::error::{Error, ErrorCode};
use crate::utils::sync::DynBase;

use super::NetChangeNotif;

/// WindowsNetifs is a type for getting all network interfaces on Windows.
///
/// A simple implementation of the NetifDiag trait, analogous to UnixNetifs.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WindowsNetifs;

impl WindowsNetifs {
    /// Get all network interfaces
    pub fn get(&self) -> Result<Vec<WindowsNetif>, Error> {
        let ifaces = NetworkInterface::show().map_err(|_| ErrorCode::NoNetworkInterface)?;

        let mut netifs: Vec<WindowsNetif> = Vec::new();

        for iface in ifaces {
            let hw_addr = parse_mac_addr(iface.mac_addr.as_deref());

            let mut ipv4addrs = Vec::new();
            let mut ipv6addrs = Vec::new();

            for addr in &iface.addr {
                match addr {
                    Addr::V4(v4) => ipv4addrs.push(v4.ip),
                    Addr::V6(v6) => ipv6addrs.push(v6.ip),
                }
            }

            // Consider the interface operational if it has at least one address
            let operational = !iface.addr.is_empty();

            netifs.push(WindowsNetif {
                name: iface.name,
                hw_addr,
                ipv4addrs,
                ipv6addrs,
                operational,
                netif_index: iface.index,
            });
        }

        Ok(netifs)
    }
}

impl DynBase for WindowsNetifs {}

impl NetifDiag for WindowsNetifs {
    fn netifs(&self, f: &mut dyn FnMut(&NetifInfo) -> Result<(), Error>) -> Result<(), Error> {
        for netif in self.get()? {
            f(&netif.to_netif_info())?;
        }

        Ok(())
    }
}

impl NetChangeNotif for WindowsNetifs {
    async fn wait_changed(&self) {
        core::future::pending().await // TODO
    }
}

/// A type for representing one network interface on Windows.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WindowsNetif {
    /// Interface name
    pub name: String,
    /// Hardware address
    pub hw_addr: [u8; 8],
    /// IPv4 addresses
    pub ipv4addrs: Vec<Ipv4Addr>,
    /// IPv6 addresses
    pub ipv6addrs: Vec<Ipv6Addr>,
    /// Operational status
    pub operational: bool,
    /// Interface index
    pub netif_index: u32,
}

impl WindowsNetif {
    /// Convert to NetifInfo
    pub fn to_netif_info(&self) -> NetifInfo<'_> {
        NetifInfo {
            name: &self.name,
            operational: self.operational,
            offprem_svc_reachable_ipv4: None,
            offprem_svc_reachable_ipv6: None,
            hw_addr: &self.hw_addr,
            ipv4_addrs: &self.ipv4addrs,
            ipv6_addrs: &self.ipv6addrs,
            netif_type: InterfaceTypeEnum::Unspecified,
            netif_index: self.netif_index,
        }
    }
}

/// Parse a MAC address string like "AA:BB:CC:DD:EE:FF" into a byte array.
fn parse_mac_addr(mac: Option<&str>) -> [u8; 8] {
    let mut result = [0u8; 8];
    if let Some(mac_str) = mac {
        for (i, part) in mac_str.split(':').enumerate() {
            if i >= 6 {
                break;
            }
            if let Ok(byte) = u8::from_str_radix(part, 16) {
                result[i] = byte;
            }
        }
    }
    result
}
