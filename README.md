# TODO more digging

 * wpa_supplicant has a mesh mode:
   * https://trac.gateworks.com/wiki/wireless/wifi/mesh
   * https://github.com/MayfieldRoboticsPublic/wpa_supplicant/blob/master/wpa_supplicant/mesh.c

# Testing WifiP2P via dbus

Gotchas:

 * Make sure the interface is not managed via networkmanager. Can be done via something like `nmcli device set wlo2 managed no`. Actually this might not be needed after all.
 * If you get dbus access errors when using wpa_supplicant's dbus interface, see which permissions are used for the messages. For example if you're using `dbus-broker`, you might need to edit something like `/usr/share/dbus-1/system.d/wpa_supplicant.conf` to grant your user (or the `wheel` group) the relevant permissions.
 * There's no chance of having multiple p2p "contexts" / interfaces per device per spec, however there can be multiple groups: https://lists.infradead.org/pipermail/hostap/2015-September/033754.html
 * I got various wpa_supplicant crashes. Was ready to submit a patch but it's fixed upstream: https://w1.fi/cgit/hostap/commit/?id=015f6a5a0cd1c8b0d40441b9fd9e4c8658bc9493
   * Instead, I submitted a request to cherry-pick: https://gitlab.archlinux.org/archlinux/packaging/packages/wpa_supplicant/-/issues/3 / https://gitlab.archlinux.org/archlinux/packaging/packages/wpa_supplicant/-/merge_requests/2
 * I got a reproducible microcode error with my Dell XPS laptop:

```
[275458.158069] iwlwifi 0000:00:14.3: Microcode SW error detected. Restarting 0x0.
[275458.158167] iwlwifi 0000:00:14.3: Start IWL Error Log Dump:
[275458.158168] iwlwifi 0000:00:14.3: Transport status: 0x0000004B, valid: 6
[275458.158169] iwlwifi 0000:00:14.3: Loaded firmware version: 89.6b44fa0b.0 so-a0-gf-a0-89.ucode
[275458.158170] iwlwifi 0000:00:14.3: 0x0000251B | ADVANCED_SYSASSERT
[275458.158171] iwlwifi 0000:00:14.3: 0x00008200 | trm_hw_status0
[275458.158171] iwlwifi 0000:00:14.3: 0x00000000 | trm_hw_status1
[275458.158172] iwlwifi 0000:00:14.3: 0x004D9CBC | branchlink2
[275458.158172] iwlwifi 0000:00:14.3: 0x004CF826 | interruptlink1
[275458.158173] iwlwifi 0000:00:14.3: 0x004CF826 | interruptlink2
[275458.158173] iwlwifi 0000:00:14.3: 0x00000001 | data1
[275458.158174] iwlwifi 0000:00:14.3: 0xDEADBEEF | data2
[275458.158174] iwlwifi 0000:00:14.3: 0xDEADBEEF | data3
[275458.158175] iwlwifi 0000:00:14.3: 0x60412682 | beacon time
[275458.158175] iwlwifi 0000:00:14.3: 0x08E369EB | tsf low
[275458.158176] iwlwifi 0000:00:14.3: 0x00000000 | tsf hi
[275458.158176] iwlwifi 0000:00:14.3: 0x00000000 | time gp1
[275458.158177] iwlwifi 0000:00:14.3: 0x08DDC466 | time gp2
[275458.158177] iwlwifi 0000:00:14.3: 0x00000001 | uCode revision type
[275458.158178] iwlwifi 0000:00:14.3: 0x00000059 | uCode version major
[275458.158179] iwlwifi 0000:00:14.3: 0x6B44FA0B | uCode version minor
[275458.158179] iwlwifi 0000:00:14.3: 0x00000370 | hw version
[275458.158180] iwlwifi 0000:00:14.3: 0x00480002 | board version
[275458.158180] iwlwifi 0000:00:14.3: 0x805FFC08 | hcmd
[275458.158180] iwlwifi 0000:00:14.3: 0x20028000 | isr0
[275458.158181] iwlwifi 0000:00:14.3: 0x00400000 | isr1
[275458.158181] iwlwifi 0000:00:14.3: 0x48F0000A | isr2
[275458.158182] iwlwifi 0000:00:14.3: 0x00C3028C | isr3
[275458.158182] iwlwifi 0000:00:14.3: 0x00000000 | isr4
[275458.158183] iwlwifi 0000:00:14.3: 0x036B001C | last cmd Id
[275458.158183] iwlwifi 0000:00:14.3: 0x00013694 | wait_event
[275458.158184] iwlwifi 0000:00:14.3: 0x00004A88 | l2p_control
[275458.158184] iwlwifi 0000:00:14.3: 0x00018034 | l2p_duration
[275458.158185] iwlwifi 0000:00:14.3: 0x00000000 | l2p_mhvalid
[275458.158185] iwlwifi 0000:00:14.3: 0x00E700D8 | l2p_addr_match
[275458.158186] iwlwifi 0000:00:14.3: 0x00000009 | lmpm_pmg_sel
[275458.158186] iwlwifi 0000:00:14.3: 0x00000000 | timestamp
[275458.158187] iwlwifi 0000:00:14.3: 0x0000D898 | flow_handler
[275458.158234] iwlwifi 0000:00:14.3: Start IWL Error Log Dump:
[275458.158235] iwlwifi 0000:00:14.3: Transport status: 0x0000004B, valid: 7
[275458.158235] iwlwifi 0000:00:14.3: 0x20000070 | NMI_INTERRUPT_LMAC_FATAL
[275458.158236] iwlwifi 0000:00:14.3: 0x00000000 | umac branchlink1
[275458.158237] iwlwifi 0000:00:14.3: 0x8048829A | umac branchlink2
[275458.158237] iwlwifi 0000:00:14.3: 0x804AC43A | umac interruptlink1
[275458.158238] iwlwifi 0000:00:14.3: 0x804AC43A | umac interruptlink2
[275458.158238] iwlwifi 0000:00:14.3: 0x00000002 | umac data1
[275458.158239] iwlwifi 0000:00:14.3: 0x804AC43A | umac data2
[275458.158239] iwlwifi 0000:00:14.3: 0x00000000 | umac data3
[275458.158240] iwlwifi 0000:00:14.3: 0x00000059 | umac major
[275458.158240] iwlwifi 0000:00:14.3: 0x6B44FA0B | umac minor
[275458.158241] iwlwifi 0000:00:14.3: 0x08DDC49C | frame pointer
[275458.158241] iwlwifi 0000:00:14.3: 0xC0886258 | stack pointer
[275458.158242] iwlwifi 0000:00:14.3: 0x0094030C | last host cmd
[275458.158242] iwlwifi 0000:00:14.3: 0x00000000 | isr status reg
[275458.158284] iwlwifi 0000:00:14.3: IML/ROM dump:
[275458.158284] iwlwifi 0000:00:14.3: 0x00000B03 | IML/ROM error/state
[275458.158292] iwlwifi 0000:00:14.3: 0x00008297 | IML/ROM data1
[275458.158300] iwlwifi 0000:00:14.3: 0x00000090 | IML/ROM WFPM_AUTH_KEY_0
[275458.158305] iwlwifi 0000:00:14.3: Fseq Registers:
[275458.158307] iwlwifi 0000:00:14.3: 0x60000000 | FSEQ_ERROR_CODE
[275458.158310] iwlwifi 0000:00:14.3: 0x803E0003 | FSEQ_TOP_INIT_VERSION
[275458.158312] iwlwifi 0000:00:14.3: 0x00190004 | FSEQ_CNVIO_INIT_VERSION
[275458.158315] iwlwifi 0000:00:14.3: 0x0000A652 | FSEQ_OTP_VERSION
[275458.158317] iwlwifi 0000:00:14.3: 0x00000003 | FSEQ_TOP_CONTENT_VERSION
[275458.158319] iwlwifi 0000:00:14.3: 0x4552414E | FSEQ_ALIVE_TOKEN
[275458.158322] iwlwifi 0000:00:14.3: 0x00080400 | FSEQ_CNVI_ID
[275458.158324] iwlwifi 0000:00:14.3: 0x00400410 | FSEQ_CNVR_ID
[275458.158326] iwlwifi 0000:00:14.3: 0x00080400 | CNVI_AUX_MISC_CHIP
[275458.158331] iwlwifi 0000:00:14.3: 0x00400410 | CNVR_AUX_MISC_CHIP
[275458.158335] iwlwifi 0000:00:14.3: 0x00009061 | CNVR_SCU_SD_REGS_SD_REG_DIG_DCDC_VTRIM
[275458.158340] iwlwifi 0000:00:14.3: 0x00000061 | CNVR_SCU_SD_REGS_SD_REG_ACTIVE_VDIG_MIRROR
[275458.158342] iwlwifi 0000:00:14.3: 0x00190004 | FSEQ_PREV_CNVIO_INIT_VERSION
[275458.158345] iwlwifi 0000:00:14.3: 0x003E0003 | FSEQ_WIFI_FSEQ_VERSION
[275458.158347] iwlwifi 0000:00:14.3: 0x003E0003 | FSEQ_BT_FSEQ_VERSION
[275458.158349] iwlwifi 0000:00:14.3: 0x000000C8 | FSEQ_CLASS_TP_VERSION
[275458.158357] iwlwifi 0000:00:14.3: UMAC CURRENT PC: 0x804abef8
[275458.158360] iwlwifi 0000:00:14.3: LMAC1 CURRENT PC: 0xd0
[275458.158453] iwlwifi 0000:00:14.3: WRT: Collecting data: ini trigger 4 fired (delay=0ms).
[275458.158457] ieee80211 phy0: Hardware restart was requested
[275458.158527] iwlwifi 0000:00:14.3: FW error in SYNC CMD STA_REMOVE_CMD
[275458.158531] CPU: 13 UID: 0 PID: 673065 Comm: wpa_supplicant Tainted: G           OE      6.12.6-arch1-1 #1 be8168881006593767299fff7299891c69c41600
[275458.158535] Tainted: [O]=OOT_MODULE, [E]=UNSIGNED_MODULE
[275458.158536] Hardware name: Dell Inc. XPS 17 9720/0W7GHH, BIOS 1.27.0 08/06/2024
[275458.158537] Call Trace:
[275458.158540]  <TASK>
[275458.158543]  dump_stack_lvl+0x5d/0x80
[275458.158551]  iwl_trans_pcie_send_hcmd+0x456/0x460 [iwlwifi 9d3434c53ab10add4e79c48332c9fc085e02208b]
[275458.158573]  ? __pfx_autoremove_wake_function+0x10/0x10
[275458.158575]  iwl_trans_send_cmd+0x53/0xf0 [iwlwifi 9d3434c53ab10add4e79c48332c9fc085e02208b]
[275458.158592]  iwl_mvm_send_cmd_pdu+0x62/0xb0 [iwlmvm 36b65c065fcf7cc6d473ce2cf89b25dcb4d73246]
[275458.158610]  iwl_mvm_mld_rm_sta_from_fw+0x49/0xb0 [iwlmvm 36b65c065fcf7cc6d473ce2cf89b25dcb4d73246]
[275458.158625]  iwl_mvm_mld_rm_int_sta+0x5a/0xc0 [iwlmvm 36b65c065fcf7cc6d473ce2cf89b25dcb4d73246]
[275458.158636]  iwl_mvm_mld_stop_ap_ibss.isra.0+0x78/0xb0 [iwlmvm 36b65c065fcf7cc6d473ce2cf89b25dcb4d73246]
[275458.158647]  ieee80211_stop_ap+0x397/0x4d0 [mac80211 b6dd9258aa3ac5a3ea59102546cf5c5acbce0260]
[275458.158693]  ___cfg80211_stop_ap+0x7f/0x2c0 [cfg80211 04508ab8e106f63b8150660d1eb49b32a2d4c345]
[275458.158739]  genl_family_rcv_msg_doit+0xef/0x150
[275458.158744]  genl_rcv_msg+0x1b7/0x2c0
[275458.158746]  ? __pfx_nl80211_pre_doit+0x10/0x10 [cfg80211 04508ab8e106f63b8150660d1eb49b32a2d4c345]
[275458.158775]  ? __pfx_nl80211_stop_ap+0x10/0x10 [cfg80211 04508ab8e106f63b8150660d1eb49b32a2d4c345]
[275458.158803]  ? __pfx_nl80211_post_doit+0x10/0x10 [cfg80211 04508ab8e106f63b8150660d1eb49b32a2d4c345]
[275458.158831]  ? __pfx_genl_rcv_msg+0x10/0x10
[275458.158833]  netlink_rcv_skb+0x50/0x100
[275458.158835]  genl_rcv+0x28/0x40
[275458.158837]  netlink_unicast+0x242/0x390
[275458.158839]  netlink_sendmsg+0x21b/0x470
[275458.158841]  ____sys_sendmsg+0x39d/0x3d0
[275458.158844]  ___sys_sendmsg+0x9a/0xe0
[275458.158846]  __sys_sendmsg+0x7a/0xd0
[275458.158849]  do_syscall_64+0x82/0x190
[275458.158852]  ? __sys_setsockopt+0x96/0xe0
[275458.158854]  ? syscall_exit_to_user_mode+0x37/0x1c0
[275458.158856]  ? do_syscall_64+0x8e/0x190
[275458.158858]  ? do_syscall_64+0x8e/0x190
[275458.158860]  ? syscall_exit_to_user_mode+0x37/0x1c0
[275458.158862]  ? do_syscall_64+0x8e/0x190
[275458.158864]  ? do_syscall_64+0x8e/0x190
[275458.158865]  ? do_syscall_64+0x8e/0x190
[275458.158867]  ? do_syscall_64+0x8e/0x190
[275458.158869]  entry_SYSCALL_64_after_hwframe+0x76/0x7e
[275458.158872] RIP: 0033:0x7f41a4f2a7f4
[275458.158907] Code: 15 21 b5 0c 00 f7 d8 64 89 02 b8 ff ff ff ff eb bf 0f 1f 44 00 00 f3 0f 1e fa 80 3d 75 38 0d 00 00 74 13 b8 2e 00 00 00 0f 05 <48> 3d 00 f0 ff ff 77 4c c3 0f 1f 00 55 48 89 e5 48 83 ec 20 89 55
[275458.158909] RSP: 002b:00007ffd189af9a8 EFLAGS: 00000202 ORIG_RAX: 000000000000002e
[275458.158911] RAX: ffffffffffffffda RBX: 00005e89511889b0 RCX: 00007f41a4f2a7f4
[275458.158912] RDX: 0000000000000000 RSI: 00007ffd189af9e0 RDI: 0000000000000006
[275458.158913] RBP: 00007ffd189af9d0 R08: 0000000000000004 R09: 000000000000000d
[275458.158914] R10: 00007ffd189afaec R11: 0000000000000202 R12: 00005e89512a2fe0
[275458.158914] R13: 00005e895118b300 R14: 00007ffd189af9e0 R15: 0000000000000000
[275458.158916]  </TASK>
[275458.158918] iwlwifi 0000:00:14.3: Failed to remove station. Id=1
[275458.158920] iwlwifi 0000:00:14.3: Failed sending remove station
[275458.722835] iwlwifi 0000:00:14.3: Failed to send LINK_CONFIG_CMD (action:2): -5
[275458.722847] iwlwifi 0000:00:14.3: Failed to send LINK_CONFIG_CMD (action:3): -5
[275458.722849] iwlwifi 0000:00:14.3: Failed to send LINK_CONFIG_CMD (action:1): -5
[275458.722854] iwlwifi 0000:00:14.3: PHY ctxt cmd error. ret=-5
[275458.723005] iwlwifi 0000:00:14.3: mcast filter cmd error. ret=-5
[275458.723009] iwlwifi 0000:00:14.3: Failed to synchronize multicast groups update
[275458.902418] iwlwifi 0000:00:14.3: WFPM_UMAC_PD_NOTIFICATION: 0x20
[275458.902466] iwlwifi 0000:00:14.3: WFPM_LMAC2_PD_NOTIFICATION: 0x1f
[275458.902518] iwlwifi 0000:00:14.3: WFPM_AUTH_KEY_0: 0x90
[275458.902568] iwlwifi 0000:00:14.3: CNVI_SCU_SEQ_DATA_DW9: 0x0
[275458.904413] iwlwifi 0000:00:14.3: RFIm is deactivated, reason = 4
```

 * I got some empty error messages for wpa_supplicant's P2P mode.
  * Fix in https://lists.infradead.org/pipermail/hostap/2025-January/043247.html

# Other links to keep track of

 * https://github.com/dbus2/zbus/issues/1180

---

 * Nice realization, that all along you can get a link-local address from a MAC address: https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/Wifi/service/java/com/android/server/wifi/p2p/WifiP2pServiceImpl.java;l=5894;drc=725fc18d701f2474328b8f21710da13d9bbb7eaf
 * But the above for some reason only gets the right group owner address, not peer address, because the peer asks the group owner for a different address (or something along those lines?).
