import { useState } from "react";
import { Layout, Menu, Avatar, Divider } from "antd";
import { useNavigate, useLocation } from "react-router-dom";
import {
  ReadOutlined,
  ScissorOutlined,
  EditOutlined,
  WifiOutlined,
  SettingOutlined,
  UserOutlined,
} from "@ant-design/icons";

const { Sider } = Layout;

export default function GlobalSidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const [collapsed, setCollapsed] = useState(true);

  // Determine which menu is selected
  const isSettingsSelected = location.pathname.startsWith("/settings");
  const selectedMainKeys = isSettingsSelected ? [] : [location.pathname];
  const selectedSettingsKeys = isSettingsSelected ? ["/settings"] : [];

  const mainMenuItems = [
    {
      key: "/papers",
      icon: <ReadOutlined />,
      label: "论文",
    },
    {
      key: "/clips",
      icon: <ScissorOutlined />,
      label: "剪藏",
    },
    {
      key: "/writing",
      icon: <EditOutlined />,
      label: "写作",
    },
    {
      key: "/subscriptions",
      icon: <WifiOutlined />,
      label: "订阅",
    },
  ];

  return (
    <Sider
      collapsible
      collapsed={collapsed}
      onCollapse={(value) => setCollapsed(value)}
      theme="light"
      width={200}
      collapsedWidth={64}
      style={{
        borderRight: "1px solid rgba(0, 0, 0, 0.06)",
        height: "100vh",
        zIndex: 100,
        // Override Ant Design's default Sider flex behavior to allow internal flex column
        // We use a wrapper div inside if Sider doesn't support flex directly well with collapse
      }}
    >
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            {/* User Avatar Section */}
            <div
                style={{
                padding: "16px 0",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                transition: "all 0.2s",
                cursor: "pointer",
                }}
                onClick={() => setCollapsed(!collapsed)}
            >
                <Avatar size={collapsed ? 32 : 48} icon={<UserOutlined />} />
                {!collapsed && (
                <div style={{ marginTop: 8, fontWeight: 500, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", width: "100%", textAlign: "center" }}>
                    User Name
                </div>
                )}
            </div>

            <Divider style={{ margin: "4px 16px 16px" }} />

            {/* Main Navigation */}
            <Menu
                mode="inline"
                selectedKeys={selectedMainKeys}
                style={{ border: "none", flex: 1 }}
                items={mainMenuItems}
                onClick={({ key }) => navigate(key)}
            />

            {/* Settings (Bottom) */}
            <div style={{ marginBottom: 48 }}> {/* Add margin for the trigger button space */}
                <Menu
                mode="inline"
                selectedKeys={selectedSettingsKeys}
                style={{ border: "none" }}
                items={[
                    {
                    key: "/settings",
                    icon: <SettingOutlined />,
                    label: "设置",
                    },
                ]}
                onClick={({ key }) => navigate(key)}
                />
            </div>
      </div>
    </Sider>
  );
}
