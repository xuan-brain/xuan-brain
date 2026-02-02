import { Layout } from "antd";
import { Outlet } from "react-router-dom";
import GlobalSidebar from "./GlobalSidebar";
import StatusBar from "./StatusBar";
import "./Layout.css";

const { Content, Footer } = Layout;

export default function MainLayout() {
  return (
    <Layout style={{ height: "100vh", overflow: "hidden" }}>
      <GlobalSidebar />
      <Layout style={{ height: "100%", overflow: "hidden" }}>
        <Content
          style={{
            flex: 1,
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
          }}
        >
          <Outlet />
        </Content>
        <Footer className="status-bar" style={{ padding: 0, flexShrink: 0 }}>
          <StatusBar />
        </Footer>
      </Layout>
    </Layout>
  );
}
