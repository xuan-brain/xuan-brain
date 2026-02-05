import { useState, useEffect } from "react";
import {
  Form,
  Input,
  Button,
  Card,
  Typography,
  message,
  List,
  Modal,
  Space,
  Tag,
} from "antd";
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  CheckOutlined,
  ApiOutlined,
} from "@ant-design/icons";

interface GrobidServer {
  id: string;
  name: string;
  url: string;
  is_active: boolean;
}

interface AppConfig {
  paper: {
    grobid: {
      servers: GrobidServer[];
    };
  };
}

async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export default function PaperSettings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingServer, setEditingServer] = useState<GrobidServer | null>(null);
  const [testing, setTesting] = useState(false);
  const [form] = Form.useForm();

  const loadConfig = async () => {
    setLoading(true);
    try {
      const data = await invokeCommand<AppConfig>("get_app_config");
      setConfig(data);
    } catch (error) {
      console.error("Failed to load config:", error);
      message.error("Failed to load configuration");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadConfig();
  }, []);

  const handleSave = async (newConfig: AppConfig) => {
    try {
      await invokeCommand("save_app_config", { config: newConfig });
      setConfig(newConfig);
      window.dispatchEvent(new CustomEvent("config-updated"));
      message.success("Configuration saved");
    } catch (error) {
      console.error("Failed to save config:", error);
      message.error("Failed to save configuration");
    }
  };

  const handleAdd = () => {
    setEditingServer(null);
    form.resetFields();
    form.setFieldsValue({ url: "https://kermitt2-grobid.hf.space" });
    setIsModalOpen(true);
  };

  const handleEdit = (server: GrobidServer) => {
    setEditingServer(server);
    form.setFieldsValue(server);
    setIsModalOpen(true);
  };

  const handleDelete = (id: string) => {
    if (!config) return;
    Modal.confirm({
      title: "Confirm Delete",
      content: "Are you sure you want to delete this server?",
      onOk: () => {
        const newServers = config.paper.grobid.servers.filter(
          (s) => s.id !== id,
        );
        handleSave({
          ...config,
          paper: {
            ...config.paper,
            grobid: { ...config.paper.grobid, servers: newServers },
          },
        });
      },
    });
  };

  const handleSetActive = (id: string) => {
    if (!config) return;
    const newServers = config.paper.grobid.servers.map((s) => ({
      ...s,
      is_active: s.id === id,
    }));
    handleSave({
      ...config,
      paper: {
        ...config.paper,
        grobid: { ...config.paper.grobid, servers: newServers },
      },
    });
  };

  const onModalOk = async () => {
    try {
      const values = await form.validateFields();
      if (!config) return;

      let newServers = config.paper.grobid.servers
        ? [...config.paper.grobid.servers]
        : [];

      if (editingServer) {
        newServers = newServers.map((s) =>
          s.id === editingServer.id ? { ...s, ...values, id: s.id } : s,
        );
      } else {
        const newServer = {
          ...values,
          id: crypto.randomUUID(),
          is_active: newServers.length === 0,
        };
        newServers.push(newServer);
      }

      await handleSave({
        ...config,
        paper: {
          ...config.paper,
          grobid: { ...config.paper.grobid, servers: newServers },
        },
      });
      setIsModalOpen(false);
    } catch (error) {
      console.error("Validation failed:", error);
    }
  };

  const testConnection = async (url: string) => {
    setTesting(true);
    try {
      const testUrl = `${url.replace(/\/$/, "")}/api/isalive`;
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000);
      const response = await fetch(testUrl, { signal: controller.signal });
      clearTimeout(timeoutId);

      if (response.ok) {
        const text = await response.text();
        if (text.trim() === "true") {
          message.success("Server is available");
        } else {
          message.warning(`Server responded: ${text}`);
        }
      } else {
        message.error(`Connection failed: ${response.status}`);
      }
    } catch (error) {
      message.error(`Connection failed: ${String(error)}`);
    } finally {
      setTesting(false);
    }
  };

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 16,
        }}
      >
        <Typography.Title level={4} style={{ margin: 0 }}>
          GROBID Servers
        </Typography.Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>
          Add Server
        </Button>
      </div>

      <List
        loading={loading}
        grid={{ gutter: 16, column: 1 }}
        dataSource={config?.paper?.grobid?.servers || []}
        renderItem={(item) => (
          <List.Item>
            <Card
              size="small"
              title={
                <Space>
                  {item.name}
                  {item.is_active && <Tag color="green">Active</Tag>}
                </Space>
              }
              extra={
                <Space>
                  {!item.is_active && (
                    <Button
                      type="text"
                      size="small"
                      icon={<CheckOutlined />}
                      onClick={() => handleSetActive(item.id)}
                      title="Set as Active"
                    />
                  )}
                  <Button
                    type="text"
                    size="small"
                    icon={<ApiOutlined />}
                    onClick={() => testConnection(item.url)}
                    title="Test Connection"
                  />
                  <Button
                    type="text"
                    size="small"
                    icon={<EditOutlined />}
                    onClick={() => handleEdit(item)}
                  />
                  <Button
                    type="text"
                    danger
                    size="small"
                    icon={<DeleteOutlined />}
                    onClick={() => handleDelete(item.id)}
                  />
                </Space>
              }
            >
              <Typography.Text type="secondary" style={{ marginRight: 8 }}>
                URL:
              </Typography.Text>
              <Typography.Text ellipsis={{ tooltip: item.url }}>
                {item.url}
              </Typography.Text>
            </Card>
          </List.Item>
        )}
      />

      <Modal
        title={editingServer ? "Edit Server" : "Add Server"}
        open={isModalOpen}
        onOk={onModalOk}
        onCancel={() => setIsModalOpen(false)}
      >
        <Form form={form} layout="vertical">
          <Form.Item
            name="name"
            label="Name"
            rules={[{ required: true, message: "Please enter a name" }]}
          >
            <Input placeholder="Local GROBID" />
          </Form.Item>
          <Form.Item
            name="url"
            label="Server URL"
            rules={[{ required: true, message: "Please enter Server URL" }]}
          >
            <Input placeholder="http://localhost:8070" />
          </Form.Item>
        </Form>
        <div style={{ textAlign: "right" }}>
          <Button
            icon={<ApiOutlined />}
            loading={testing}
            onClick={() => {
              const url = form.getFieldValue("url");
              if (url) testConnection(url);
              else message.warning("Enter URL first");
            }}
          >
            Test URL
          </Button>
        </div>
      </Modal>
    </div>
  );
}
