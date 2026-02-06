import React, { useState, useEffect, useCallback } from "react";
import { Table, Text, Badge, Group, Stack, rem } from "@mantine/core";
import { IconFile } from "@tabler/icons-react";
import {
  CaretRightOutlined,
  DeleteOutlined,
  UndoOutlined,
  ExclamationCircleOutlined,
  PaperClipOutlined,
  FolderOpenOutlined,
} from "@ant-design/icons";
import { Dropdown, type MenuProps, Modal } from "antd";
import { open } from "@tauri-apps/plugin-dialog";
import { useNavigate } from "react-router-dom";
import { useI18n } from "../../lib/i18n";
import { useAppStore } from "../../stores/useAppStore";
import { Tag } from "antd";
import DocumentToolbar from "./DocumentToolbar";

async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

const TAG_COLORS: Record<string, string> = {
  red: "#ef4444",
  orange: "#f97316",
  amber: "#f59e0b",
  yellow: "#eab308",
  lime: "#84cc16",
  green: "#22c55e",
  emerald: "#10b981",
  teal: "#14b8a6",
  cyan: "#06b6d4",
  sky: "#0ea5e9",
  blue: "#3b82f6",
  indigo: "#6366f1",
  violet: "#8b5cf6",
  purple: "#a855f7",
  fuchsia: "#d946ef",
  pink: "#ec4899",
  rose: "#f43f5e",
};

const TABLE_FONT_SIZE = 13;

function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

interface LabelDto {
  id: number;
  name: string;
  color: string;
}

interface AttachmentDto {
  id: number;
  paper_id: number;
  file_name: string;
  file_type: string;
  created_at: string;
}

interface PaperDto {
  id: number;
  title: string;
  publication_year?: number;
  journal_name?: string;
  conference_name?: string;
  authors: string[];
  labels: LabelDto[];
  attachment_count?: number;
}

interface DocumentListProps {
  onDocumentSelect: (document: any) => void;
  categoryId?: string | null;
}

const AttachmentList = ({ paperId }: { paperId: number }) => {
  const [attachments, setAttachments] = useState<AttachmentDto[]>([]);
  const [loading, setLoading] = useState(true);

  const loadAttachments = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invokeCommand<AttachmentDto[]>("get_attachments", {
        paperId: paperId,
      });
      setAttachments(data);
    } catch (error) {
      console.error("Failed to load attachments:", error);
    } finally {
      setLoading(false);
    }
  }, [paperId]);

  useEffect(() => {
    loadAttachments();

    const handleUpdate = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (detail && detail.paperId === paperId) {
        loadAttachments();
      }
    };

    window.addEventListener("attachment-updated", handleUpdate);
    return () => window.removeEventListener("attachment-updated", handleUpdate);
  }, [loadAttachments, paperId]);

  if (loading)
    return (
      <Text size="sm" c="dimmed">
        Loading...
      </Text>
    );
  if (attachments.length === 0)
    return (
      <Text size="sm" c="dimmed">
        No attachments
      </Text>
    );

  return (
    <div style={{ padding: rem(8) }}>
      <Stack gap={rem(8)}>
        {attachments.map((att) => (
          <Group
            key={att.id}
            p={rem(8)}
            bg="gray.1"
            style={{
              borderRadius: rem(4),
              whiteSpace: "nowrap",
            }}
          >
            <IconFile size={16} />
            <Text size="sm" flex={1} style={{ whiteSpace: "nowrap" }}>
              {att.file_name}
            </Text>
            <Text size="xs" c="dimmed">
              {new Date(att.created_at).toLocaleDateString()}
            </Text>
          </Group>
        ))}
      </Stack>
    </div>
  );
};

export default function DocumentListMantine({
  onDocumentSelect,
  categoryId,
}: DocumentListProps) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { selectedDocument, accentColor, isDark } = useAppStore();
  const [rows, setRows] = useState<PaperDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [openedRows, setOpenedRows] = useState<Set<number>>(new Set());
  const [hoveredRowId, setHoveredRowId] = useState<number | null>(null);

  const loadPapers = useCallback(async () => {
    setLoading(true);
    try {
      let papers: PaperDto[];
      if (categoryId === "trash") {
        papers = await invokeCommand<PaperDto[]>("get_deleted_papers");
      } else if (categoryId) {
        papers = await invokeCommand<PaperDto[]>("get_papers_by_category", {
          categoryPath: categoryId,
        });
      } else {
        papers = await invokeCommand<PaperDto[]>("get_all_papers");
      }
      setRows(papers);
      if (papers.length > 0) {
        onDocumentSelect(papers[0]);
      }
    } catch (error) {
      console.error("Failed to load papers:", error);
      const demoData = [
        {
          id: 1,
          title: "Attention Is All You Need",
          authors: ["Vaswani et al."],
          publication_year: 2017,
          conference_name: "NIPS",
          labels: [],
        },
      ];
      setRows(demoData);
      if (demoData.length > 0) {
        onDocumentSelect(demoData[0]);
      }
    } finally {
      setLoading(false);
    }
  }, [categoryId, onDocumentSelect]);

  useEffect(() => {
    loadPapers();

    const handlePaperUpdate = (e: Event) => {
      const detail = (e as CustomEvent).detail as PaperDto;
      setRows((prev) =>
        prev.map((row) => (row.id === detail.id ? { ...row, ...detail } : row)),
      );
    };

    window.addEventListener("paper-updated", handlePaperUpdate);
    return () => window.removeEventListener("paper-updated", handlePaperUpdate);
  }, [loadPapers]);

  const handleDoubleClick = useCallback(
    (record: PaperDto) => {
      console.info("Double clicked paper:", record.id, record.title);
      navigate(`/papers/${record.id}`);
    },
    [navigate],
  );

  const handleRowClick = useCallback(
    (record: PaperDto) => {
      console.info("Row clicked", record.id);
      onDocumentSelect(record);
    },
    [onDocumentSelect],
  );

  const handleRowExpand = useCallback((id: number) => {
    setOpenedRows((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const handleAddAttachment = useCallback(
    async (paperId: number) => {
      try {
        const selected = await open({
          multiple: false,
          directory: false,
        });
        if (selected) {
          const filePath = Array.isArray(selected) ? selected[0] : selected;
          if (filePath) {
            await invokeCommand("add_attachment", {
              paperId: paperId,
              filePath: filePath,
            });
            window.dispatchEvent(
              new CustomEvent("attachment-updated", {
                detail: { paperId: paperId },
              }),
            );
            await loadPapers();
            Modal.success({ content: "Attachment added successfully" });
          }
        }
      } catch (error) {
        console.error("Failed to add attachment:", error);
        Modal.error({
          title: "Failed to add attachment",
          content: String(error),
        });
      }
    },
    [loadPapers],
  );

  const handleOpenFolder = useCallback(async (paperId: number) => {
    try {
      await invokeCommand("open_paper_folder", { paperId: paperId });
    } catch (error) {
      console.error("Failed to open folder:", error);
      Modal.error({
        title: "Failed to open folder",
        content: String(error),
      });
    }
  }, []);

  const handleDeletePaper = useCallback(
    async (paperId: number) => {
      Modal.confirm({
        title: t("dialog.delete"),
        icon: <ExclamationCircleOutlined />,
        content: "确定要删除此文档吗？此操作将把文档移入回收站。",
        okText: t("dialog.delete"),
        okType: "danger",
        cancelText: t("dialog.cancel"),
        onOk: async () => {
          try {
            await invokeCommand("delete_paper", { id: paperId });
            await loadPapers();
          } catch (error) {
            console.error("Failed to delete paper:", error);
            Modal.error({
              title: t("dialog.deleteFailed"),
              content: String(error),
            });
          }
        },
      });
    },
    [t, loadPapers],
  );

  const handleRestorePaper = useCallback(
    async (paperId: number) => {
      try {
        await invokeCommand("restore_paper", { id: paperId });
        await loadPapers();
      } catch (error) {
        console.error("Failed to restore paper:", error);
        Modal.error({
          title: t("dialog.restoreFailed"),
          content: String(error),
        });
      }
    },
    [t, loadPapers],
  );

  const handlePermanentlyDelete = useCallback(
    async (paperId: number) => {
      Modal.confirm({
        title: t("dialog.permanentlyDelete"),
        icon: <ExclamationCircleOutlined />,
        content: t("dialog.confirmPermanentlyDelete"),
        okText: t("dialog.permanentlyDelete"),
        okType: "danger",
        cancelText: t("dialog.cancel"),
        onOk: async () => {
          try {
            await invokeCommand("permanently_delete_paper", {
              id: paperId,
            });
            await loadPapers();
          } catch (error) {
            console.error("Failed to delete paper:", error);
            Modal.error({
              title: t("dialog.deleteFailed"),
              content: String(error),
            });
          }
        },
      });
    },
    [t, loadPapers],
  );

  return (
    <div
      style={{
        height: "100%",
        width: "100%",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <DocumentToolbar onRefresh={loadPapers} selectedCategoryId={categoryId} />

      <div
        style={{
          flex: 1,
          overflowY: "auto",
          overflowX: "hidden",
          minHeight: 0,
          marginTop: 16,
        }}
      >
        {loading ? (
          <div
            style={{
              display: "flex",
              justifyContent: "center",
              alignItems: "center",
              height: "100%",
            }}
          >
            <Text c="dimmed">Loading papers...</Text>
          </div>
        ) : (
          <Table
            highlightOnHover
            verticalSpacing="sm"
            horizontalSpacing="md"
            style={{ width: "100%", tableLayout: "fixed" }}
          >
            <Table.Thead>
              <Table.Tr>
                <Table.Th
                  style={{
                    width: "40px",
                    textAlign: "center",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                ></Table.Th>
                <Table.Th
                  style={{
                    width: "80%",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                >
                  {t("document.title")}
                </Table.Th>
                <Table.Th
                  style={{
                    width: "100px",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                >
                  {t("document.authors")}
                </Table.Th>
                <Table.Th
                  style={{
                    width: "200px",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                >
                  {t("document.source")}
                </Table.Th>
                <Table.Th
                  style={{
                    width: "30px",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                >
                  {t("document.year")}
                </Table.Th>
                <Table.Th
                  style={{
                    width: "100px",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                >
                  {t("document.labels")}
                </Table.Th>
              </Table.Tr>
            </Table.Thead>

            <Table.Tbody>
              {rows.map((record, index) => {
                const isSelected = selectedDocument?.id === record.id;
                const isExpanded = openedRows.has(record.id);
                const isOddRow = index % 2 !== 0;

                const menuItems: MenuProps["items"] =
                  categoryId === "trash"
                    ? [
                        {
                          key: "restore",
                          label: t("dialog.restore"),
                          icon: <UndoOutlined />,
                          onClick: () => handleRestorePaper(record.id),
                        },
                        {
                          key: "permanently_delete",
                          label: t("dialog.permanentlyDelete"),
                          icon: <DeleteOutlined />,
                          danger: true,
                          onClick: () => handlePermanentlyDelete(record.id),
                        },
                      ]
                    : [
                        {
                          key: "add_attachment",
                          label: "添加附件",
                          icon: <PaperClipOutlined />,
                          onClick: () => handleAddAttachment(record.id),
                        },
                        {
                          key: "open_folder",
                          label: "打开附件文件夹",
                          icon: <FolderOpenOutlined />,
                          onClick: () => handleOpenFolder(record.id),
                        },
                        {
                          type: "divider",
                        },
                        {
                          key: "delete",
                          label: t("dialog.delete"),
                          icon: <DeleteOutlined />,
                          danger: true,
                          onClick: () => handleDeletePaper(record.id),
                        },
                      ];

                return (
                  <React.Fragment key={record.id}>
                    <Dropdown
                      menu={{ items: menuItems }}
                      trigger={["contextMenu"]}
                    >
                      <Table.Tr
                        style={{
                          cursor: "pointer",
                          whiteSpace: "nowrap",
                          "--table-accent-color-transparent": hexToRgba(
                            accentColor,
                            0.5,
                          ),
                        }}
                        data-striped={isOddRow}
                        data-selected={isSelected}
                        onDoubleClick={() => handleDoubleClick(record)}
                        onClick={() => handleRowClick(record)}
                        onMouseEnter={() => setHoveredRowId(record.id)}
                        onMouseLeave={() => setHoveredRowId(null)}
                      >
                        <Table.Td
                          style={{
                            width: "40px",
                            textAlign: "center",
                            verticalAlign: "middle",
                          }}
                          onClick={(e) => {
                            e.stopPropagation();
                            if (
                              record.attachment_count &&
                              record.attachment_count > 0
                            ) {
                              handleRowExpand(record.id);
                            }
                          }}
                        >
                          {typeof record.attachment_count === "number" &&
                            record.attachment_count > 0 && (
                              <div
                                style={{
                                  cursor: "pointer",
                                  display: "flex",
                                  alignItems: "center",
                                  justifyContent: "center",
                                }}
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleRowExpand(record.id);
                                }}
                              >
                                <CaretRightOutlined
                                  style={{
                                    fontSize: "14px",
                                    color: isDark
                                      ? "rgba(255, 255, 255, 0.65)"
                                      : "rgba(0, 0, 0, 0.45)",
                                  }}
                                />
                              </div>
                            )}
                        </Table.Td>
                        <Table.Td
                          style={{
                            width: "1",
                            verticalAlign: "middle",
                            paddingLeft: 8,
                            paddingRight: 8,
                          }}
                        >
                          <Text
                            size="sm"
                            truncate="end"
                            title={record.title}
                            style={{
                              whiteSpace: "nowrap",
                              overflow: "hidden",
                              textOverflow: "ellipsis",
                              fontSize: `${TABLE_FONT_SIZE}px`,
                              lineHeight: 1,
                              margin: 0,
                              padding: 0,
                              display: "block",
                              maxWidth: "100%",
                            }}
                          >
                            {record.title}
                          </Text>
                        </Table.Td>
                        <Table.Td
                          style={{
                            width: "100px",
                            verticalAlign: "middle",
                          }}
                        >
                          <div
                            style={{
                              display: "inline-flex",
                              alignItems: "center",
                              gap: "4px",
                              flexWrap:
                                hoveredRowId === record.id ? "wrap" : "nowrap",
                              height: "100%",
                            }}
                          >
                            {hoveredRowId === record.id
                              ? record.authors?.map((author, index) => (
                                  <Tag
                                    key={index}
                                    color="blue"
                                    style={{
                                      whiteSpace: "nowrap",
                                      fontSize: `${TABLE_FONT_SIZE}px`,
                                      margin: 0,
                                      padding: "2px 6px",
                                      lineHeight: 1,
                                    }}
                                  >
                                    {author}
                                  </Tag>
                                ))
                              : record.authors
                                  ?.slice(0, 1)
                                  .map((author, index) => (
                                    <Tag
                                      key={index}
                                      color="blue"
                                      style={{
                                        whiteSpace: "nowrap",
                                        fontSize: `${TABLE_FONT_SIZE}px`,
                                        margin: 0,
                                        padding: "2px 6px",
                                        lineHeight: 1,
                                        position: "relative",
                                      }}
                                    >
                                      {author}
                                      {record.authors.length > 1 && (
                                        <Badge
                                          variant="filled"
                                          size="xs"
                                          style={{
                                            fontSize: `${TABLE_FONT_SIZE - 2}px`,
                                            height: 14,
                                            minWidth: 14,
                                            display: "inline-flex",
                                            alignItems: "center",
                                            justifyContent: "center",
                                            padding: "0 5px",
                                            borderRadius: 3,
                                            backgroundColor: "#ef4444",
                                            color: "white",
                                            position: "absolute",
                                            top: -6,
                                            right: -6,
                                            zIndex: 1,
                                          }}
                                        >
                                          +{record.authors.length - 1}
                                        </Badge>
                                      )}
                                    </Tag>
                                  ))}
                          </div>
                        </Table.Td>
                        <Table.Td
                          style={{
                            width: "250px",
                            verticalAlign: "middle",
                            textAlign: "center",
                          }}
                        >
                          <Text
                            size="sm"
                            truncate="end"
                            title={
                              record.journal_name ||
                              record.conference_name ||
                              ""
                            }
                            style={{
                              whiteSpace: "nowrap",
                              fontSize: `${TABLE_FONT_SIZE}px`,
                              lineHeight: 1,
                              margin: 0,
                              padding: 0,
                            }}
                          >
                            {record.journal_name ||
                              record.conference_name ||
                              ""}
                          </Text>
                        </Table.Td>
                        <Table.Td
                          style={{
                            width: "25px",
                            verticalAlign: "middle",
                            textAlign: "center",
                          }}
                        >
                          <Text
                            size="sm"
                            style={{
                              whiteSpace: "nowrap",
                              fontSize: `${TABLE_FONT_SIZE}px`,
                              lineHeight: 1,
                              margin: 0,
                              padding: 0,
                            }}
                          >
                            {record.publication_year
                              ? String(record.publication_year)
                              : ""}
                          </Text>
                        </Table.Td>
                        <Table.Td
                          style={{
                            width: "50px",
                            verticalAlign: "middle",
                            textAlign: "center",
                          }}
                        >
                          <div
                            style={{
                              display: "inline-flex",
                              alignItems: "center",
                              gap: "4px",
                              flexWrap:
                                hoveredRowId === record.id ? "wrap" : "nowrap",
                              height: "100%",
                            }}
                          >
                            {hoveredRowId === record.id
                              ? record.labels?.map((label) => (
                                  <Tag
                                    key={label.id}
                                    color={
                                      TAG_COLORS[label.color] || TAG_COLORS.blue
                                    }
                                    style={{
                                      whiteSpace: "nowrap",
                                      fontSize: `${TABLE_FONT_SIZE}px`,
                                      margin: 0,
                                      padding: "2px 6px",
                                      lineHeight: 1,
                                    }}
                                  >
                                    {label.name}
                                  </Tag>
                                ))
                              : record.labels?.slice(0, 1).map((label) => (
                                  <Tag
                                    key={label.id}
                                    color={
                                      TAG_COLORS[label.color] || TAG_COLORS.blue
                                    }
                                    style={{
                                      whiteSpace: "nowrap",
                                      fontSize: `${TABLE_FONT_SIZE}px`,
                                      margin: 0,
                                      padding: "2px 6px",
                                      lineHeight: 1,
                                      position: "relative",
                                    }}
                                  >
                                    {label.name}
                                    {record.labels.length > 1 && (
                                      <Badge
                                        variant="filled"
                                        size="xs"
                                        style={{
                                          fontSize: `${TABLE_FONT_SIZE - 2}px`,
                                          height: 14,
                                          minWidth: 14,
                                          display: "inline-flex",
                                          alignItems: "center",
                                          justifyContent: "center",
                                          padding: "0 5px",
                                          borderRadius: 3,
                                          backgroundColor: "#ef4444",
                                          color: "white",
                                          position: "absolute",
                                          top: -6,
                                          right: -6,
                                          zIndex: 1,
                                        }}
                                      >
                                        +{record.labels.length - 1}
                                      </Badge>
                                    )}
                                  </Tag>
                                ))}
                          </div>
                        </Table.Td>
                      </Table.Tr>
                    </Dropdown>

                    {isExpanded && (
                      <Table.Tr>
                        <Table.Td colSpan={6} style={{ padding: 0 }}>
                          <div
                            style={{
                              padding: rem(8),
                              backgroundColor: "var(--mantine-color-gray-0)",
                              borderTop:
                                "1px solid var(--mantine-color-gray-2)",
                            }}
                          >
                            <AttachmentList paperId={record.id} />
                          </div>
                        </Table.Td>
                      </Table.Tr>
                    )}
                  </React.Fragment>
                );
              })}
            </Table.Tbody>
          </Table>
        )}
      </div>
    </div>
  );
}
