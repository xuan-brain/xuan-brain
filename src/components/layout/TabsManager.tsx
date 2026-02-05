import { FC } from "react";
import { Tabs, Dropdown, type MenuProps } from "antd";
import { CloseOutlined } from "@ant-design/icons";
import { useNavigate } from "react-router-dom";
import { useTabsStore } from "@/stores/useTabsStore";
import "./TabsManager.css";

const TabsManager: FC = () => {
  const navigate = useNavigate();
  const {
    tabs,
    activeTabId,
    removeTab,
    setActiveTab,
    closeOtherTabs,
    closeAllTabs,
  } = useTabsStore();

  // Handle tab click
  const handleTabClick = (tabId: string) => {
    const tab = tabs.find((t) => t.id === tabId);
    if (tab) {
      setActiveTab(tabId);
      navigate(tab.path);
    }
  };

  // Handle tab close
  const handleTabClose = (e: React.MouseEvent, tabId: string) => {
    e.stopPropagation();

    removeTab(tabId);

    // Navigate away if closing active tab
    if (tabId === activeTabId) {
      const remainingTabs = tabs.filter((t) => t.id !== tabId);
      if (remainingTabs.length > 0) {
        // Activate and navigate to last tab
        const lastTab = remainingTabs[remainingTabs.length - 1];
        setActiveTab(lastTab.id);
        navigate(lastTab.path);
      } else {
        // No tabs left, navigate back to papers list
        navigate("/papers");
      }
    }
  };

  // Context menu items
  const getContextMenuItems = (tabId: string): MenuProps["items"] => {
    return [
      {
        key: "close",
        label: "Close",
        onClick: (e) => {
          e.domEvent.stopPropagation();
          handleTabClose(e.domEvent as React.MouseEvent, tabId);
        },
      },
      {
        key: "close-others",
        label: "Close Others",
        onClick: (e) => {
          e.domEvent.stopPropagation();
          closeOtherTabs(tabId);
          const tab = tabs.find((t) => t.id === tabId);
          if (tab) navigate(tab.path);
        },
      },
      {
        key: "close-all",
        label: "Close All",
        onClick: (e) => {
          e.domEvent.stopPropagation();
          closeAllTabs();
          navigate("/papers");
        },
      },
    ];
  };

  // If no tabs, don't render
  if (tabs.length === 0) {
    return null;
  }

  return (
    <div className="tabs-manager">
      <Tabs
        activeKey={activeTabId || undefined}
        type="editable-card"
        hideAdd
        onTabClick={handleTabClick}
        items={tabs.map((tab) => {
          const contextMenuItems = getContextMenuItems(tab.id);

          return {
            key: tab.id,
            label: (
              <Dropdown
                menu={{ items: contextMenuItems }}
                trigger={["contextMenu"]}
              >
                <span className="tab-label" title={tab.title}>
                  {tab.title}
                </span>
              </Dropdown>
            ),
            closeIcon: (
              <CloseOutlined
                className="tab-close-icon"
                onClick={(e) => handleTabClose(e, tab.id)}
              />
            ),
          };
        })}
      />
    </div>
  );
};

export default TabsManager;
