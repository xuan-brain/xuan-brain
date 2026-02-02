import { useState } from "react";
import { Tabs, Typography } from "antd";
import {
  UserOutlined,
  DesktopOutlined,
  ReadOutlined,
  ScissorOutlined,
  EditOutlined,
  WifiOutlined,
  AppstoreOutlined,
} from "@ant-design/icons";
import SystemSettings from "../components/settings/SystemSettings";

const { Title } = Typography;

export default function SettingsPage() {
  const [activeKey, setActiveKey] = useState("system");

  const items = [
    {
      key: "user",
      label: "用户",
      icon: <UserOutlined />,
      children: <div>用户设置（待开发）</div>,
    },
    {
      key: "system",
      label: "系统",
      icon: <DesktopOutlined />,
      children: <SystemSettings />,
    },
    {
      key: "papers",
      label: "论文",
      icon: <ReadOutlined />,
      children: <div>论文设置（待开发）</div>,
    },
    {
      key: "clips",
      label: "剪藏",
      icon: <ScissorOutlined />,
      children: <div>剪藏设置（待开发）</div>,
    },
    {
      key: "writing",
      label: "写作",
      icon: <EditOutlined />,
      children: <div>写作设置（待开发）</div>,
    },
    {
      key: "subscriptions",
      label: "订阅",
      icon: <WifiOutlined />,
      children: <div>订阅设置（待开发）</div>,
    },
    {
      key: "other",
      label: "其他",
      icon: <AppstoreOutlined />,
      children: <div>其他设置（待开发）</div>,
    },
  ];

  return (
    <div style={{ height: "100%", padding: "24px" }}>
      <Title level={2} style={{ marginBottom: 24 }}>
        设置
      </Title>
      <Tabs
        tabPosition="left"
        activeKey={activeKey}
        onChange={setActiveKey}
        items={items.map((item) => ({
          key: item.key,
          label: (
            <span>
              {item.icon}
              <span style={{ marginLeft: 8 }}>{item.label}</span>
            </span>
          ),
          children: (
            <div style={{ paddingLeft: 24, maxWidth: 800 }}>
              {item.children}
            </div>
          ),
        }))}
        style={{ height: "100%" }}
      />
    </div>
  );
}
