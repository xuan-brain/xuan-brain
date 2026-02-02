import { useState, useEffect } from "react";
import {
  Form,
  Input,
  Button,
  List,
  Modal,
  Card,
  Typography,
  Space,
  Tag,
  message,
} from "antd";
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  CheckOutlined,
} from "@ant-design/icons";

// We'll define interfaces here for now, eventually move to a shared types file
interface LlmProvider {
  id: string;
  name: string;
  api_key: string;
  base_url: string;
  model_name: string;
  is_default: boolean;
}

interface AppConfig {
  system: {
    llm_providers: LlmProvider[];
  };
}

async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export default function SystemSettings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingProvider, setEditingProvider] = useState<LlmProvider | null>(
    null,
  );
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
      message.success("Configuration saved");
    } catch (error) {
      console.error("Failed to save config:", error);
      message.error("Failed to save configuration");
    }
  };

  const handleAdd = () => {
    setEditingProvider(null);
    form.resetFields();
    form.setFieldsValue({ base_url: "https://api.openai.com/v1" });
    setIsModalOpen(true);
  };

  const handleEdit = (provider: LlmProvider) => {
    setEditingProvider(provider);
    form.setFieldsValue(provider);
    setIsModalOpen(true);
  };

  const handleDelete = (id: string) => {
    if (!config) return;
    Modal.confirm({
      title: "Confirm Delete",
      content: "Are you sure you want to delete this provider?",
      onOk: () => {
        const newProviders = config.system.llm_providers.filter(
          (p) => p.id !== id,
        );
        handleSave({ ...config, system: { ...config.system, llm_providers: newProviders } });
      },
    });
  };

  const handleSetDefault = (id: string) => {
    if (!config) return;
    const newProviders = config.system.llm_providers.map((p) => ({
      ...p,
      is_default: p.id === id,
    }));
    handleSave({ ...config, system: { ...config.system, llm_providers: newProviders } });
  };

  const onModalOk = async () => {
    try {
      const values = await form.validateFields();
      if (!config) return;

      let newProviders = [...config.system.llm_providers];

      if (editingProvider) {
        // Edit
        newProviders = newProviders.map((p) =>
          p.id === editingProvider.id ? { ...p, ...values, id: p.id } : p,
        );
      } else {
        // Add
        const newProvider = {
          ...values,
          id: crypto.randomUUID(),
          is_default: newProviders.length === 0, // First one is default
        };
        newProviders.push(newProvider);
      }

      await handleSave({ ...config, system: { ...config.system, llm_providers: newProviders } });
      setIsModalOpen(false);
    } catch (error) {
      console.error("Validation failed:", error);
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
          LLM Providers
        </Typography.Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>
          Add Provider
        </Button>
      </div>

      <List
        loading={loading}
        grid={{ gutter: 16, column: 1 }}
        dataSource={config?.system.llm_providers || []}
        renderItem={(item) => (
          <List.Item>
            <Card
              size="small"
              title={
                <Space>
                  {item.name}
                  {item.is_default && <Tag color="green">Default</Tag>}
                </Space>
              }
              extra={
                <Space>
                  {!item.is_default && (
                    <Button
                      type="text"
                      size="small"
                      icon={<CheckOutlined />}
                      onClick={() => handleSetDefault(item.id)}
                      title="Set as Default"
                    />
                  )}
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
              <div style={{ display: "grid", gridTemplateColumns: "auto 1fr", gap: "8px 16px" }}>
                <Typography.Text type="secondary">Model:</Typography.Text>
                <Typography.Text>{item.model_name}</Typography.Text>

                <Typography.Text type="secondary">Base URL:</Typography.Text>
                <Typography.Text ellipsis={{ tooltip: item.base_url }}>{item.base_url}</Typography.Text>
              </div>
            </Card>
          </List.Item>
        )}
      />

      <Modal
        title={editingProvider ? "Edit Provider" : "Add Provider"}
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
            <Input placeholder="e.g. OpenAI, DeepSeek" />
          </Form.Item>
          <Form.Item
            name="base_url"
            label="Base URL"
            rules={[{ required: true, message: "Please enter Base URL" }]}
          >
            <Input placeholder="https://api.openai.com/v1" />
          </Form.Item>
          <Form.Item
            name="api_key"
            label="API Key"
            rules={[{ required: true, message: "Please enter API Key" }]}
          >
            <Input.Password placeholder="sk-..." />
          </Form.Item>
          <Form.Item
            name="model_name"
            label="Model Name"
            rules={[{ required: true, message: "Please enter Model Name" }]}
          >
            <Input placeholder="gpt-4o, deepseek-chat" />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
