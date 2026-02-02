import { useState, useEffect } from "react";
import { Select, message } from "antd";
import { DeploymentUnitOutlined } from "@ant-design/icons";

interface LlmProvider {
  id: string;
  name: string;
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

export default function LlmSwitcher() {
  const [providers, setProviders] = useState<LlmProvider[]>([]);
  const [currentProviderId, setCurrentProviderId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const config = await invokeCommand<AppConfig>("get_app_config");
      const list = config.system.llm_providers || [];
      setProviders(list);
      const defaultProvider = list.find((p) => p.is_default);
      if (defaultProvider) {
        setCurrentProviderId(defaultProvider.id);
      } else if (list.length > 0) {
        // Fallback to first if no default
        setCurrentProviderId(list[0].id);
      } else {
        setCurrentProviderId(null);
      }
    } catch (error) {
      console.error("Failed to load LLM config:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadConfig();

    const handleConfigUpdate = () => {
      loadConfig();
    };

    window.addEventListener("config-updated", handleConfigUpdate);
    return () => window.removeEventListener("config-updated", handleConfigUpdate);
  }, []);

  const handleChange = async (value: string) => {
    try {
      const config = await invokeCommand<AppConfig>("get_app_config");
      const newProviders = config.system.llm_providers.map((p) => ({
        ...p,
        is_default: p.id === value,
      }));

      await invokeCommand("save_app_config", {
        config: { ...config, system: { ...config.system, llm_providers: newProviders } },
      });

      setCurrentProviderId(value);
      setProviders(newProviders);
      message.success("Default LLM provider updated");

      window.dispatchEvent(new CustomEvent("config-updated"));

    } catch (error) {
      console.error("Failed to update default provider:", error);
      message.error("Failed to update provider");
    }
  };

  if (providers.length === 0) {
    return (
        <div style={{ fontSize: 12, color: '#999', display: 'flex', alignItems: 'center', gap: 4 }}>
            <DeploymentUnitOutlined />
            <span>No LLM</span>
        </div>
    );
  }

  return (
    <Select
      value={currentProviderId}
      onChange={handleChange}
      size="small"
      style={{ width: 140, fontSize: 12 }}
      bordered={false}
      loading={loading}
      suffixIcon={<DeploymentUnitOutlined style={{ fontSize: 12 }} />}
      options={providers.map((p) => ({ label: <span style={{ fontSize: 12 }}>{p.name}</span>, value: p.id }))}
    />
  );
}
