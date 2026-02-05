import { useState, useEffect } from "react";
import { Select, message } from "antd";
import { ApiOutlined } from "@ant-design/icons";

interface GrobidServer {
  id: string;
  name: string;
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

export default function GrobidSwitcher() {
  const [servers, setServers] = useState<GrobidServer[]>([]);
  const [currentServerId, setCurrentServerId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const config = await invokeCommand<AppConfig>("get_app_config");
      const list = config.paper?.grobid?.servers || [];
      setServers(list);
      const activeServer = list.find((s) => s.is_active);
      if (activeServer) {
        setCurrentServerId(activeServer.id);
      } else if (list.length > 0) {
        setCurrentServerId(list[0].id);
      } else {
        setCurrentServerId(null);
      }
    } catch (error) {
      console.error("Failed to load GROBID config:", error);
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
      const currentServers = config.paper?.grobid?.servers || [];
      const newServers = currentServers.map((s) => ({
        ...s,
        is_active: s.id === value,
      }));

      const newConfig = {
        ...config,
        paper: {
          ...config.paper,
          grobid: {
            ...config.paper?.grobid,
            servers: newServers,
          }
        }
      };

      await invokeCommand("save_app_config", { config: newConfig });

      setCurrentServerId(value);
      setServers(newServers);
      message.success("Active GROBID server updated");

      window.dispatchEvent(new CustomEvent("config-updated"));

    } catch (error) {
      console.error("Failed to update active server:", error);
      message.error("Failed to update server");
    }
  };

  if (servers.length === 0) {
    return (
        <div style={{ fontSize: 12, color: '#999', display: 'flex', alignItems: 'center', gap: 4 }}>
            <ApiOutlined />
            <span>No GROBID</span>
        </div>
    );
  }

  return (
    <Select
      value={currentServerId}
      onChange={handleChange}
      size="small"
      style={{ width: 140, fontSize: 12 }}
      bordered={false}
      loading={loading}
      suffixIcon={<ApiOutlined style={{ fontSize: 12 }} />}
      options={servers.map((s) => ({ label: <span style={{ fontSize: 12 }}>{s.name}</span>, value: s.id }))}
    />
  );
}
