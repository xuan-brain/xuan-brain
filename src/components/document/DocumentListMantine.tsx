import React, { useState, useEffect, useCallback } from "react";
import {
  Table,
  ActionIcon,
  Menu,
  Text,
  Badge,
  Group,
  Stack,
  rem,
  MenuTarget,
  MenuDropdown,
  MenuLabel,
  MenuItem,
  MenuDivider,
} from "@mantine/core";
import {
  IconFile,
  IconTrash,
  IconRestore,
  IconDots,
  IconPaperclip,
  IconFolderOpen,
  IconGripHorizontal,
} from "@tabler/icons-react";
import { useNavigate } from "react-router-dom";
import { useI18n } from "../../lib/i18n";
import { useAppStore } from "../../stores/useAppStore";
import { open } from "@tauri-apps/plugin-dialog";
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

// Storage key for column widths
const COLUMN_WIDTHS_KEY = "document-list-column-widths";

interface ColumnWidths {
  title: number;
  authors: number;
  source: number;
  year: number;
  labels: number;
}

const DEFAULT_COLUMN_WIDTHS: ColumnWidths = {
  title: 250,
  authors: 200,
  source: 150,
  year: 80,
  labels: 320,
};

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
  const { selectedDocument, accentColor } = useAppStore();
  const [rows, setRows] = useState<PaperDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [openedRows, setOpenedRows] = useState<Set<number>>(new Set());

  // Load and save column widths
  const [columnWidths, setColumnWidths] = useState<ColumnWidths>(() => {
    const saved = localStorage.getItem(COLUMN_WIDTHS_KEY);
    if (saved) {
      try {
        return JSON.parse(saved) as ColumnWidths;
      } catch (e) {
        console.error("Failed to parse column widths:", e);
        return DEFAULT_COLUMN_WIDTHS;
      }
    }
    return DEFAULT_COLUMN_WIDTHS;
  });

  const saveColumnWidths = useCallback((widths: ColumnWidths) => {
    try {
      localStorage.setItem(COLUMN_WIDTHS_KEY, JSON.stringify(widths));
    } catch (e) {
      console.error("Failed to save column widths:", e);
    }
  }, []);

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

  const handleDelete = useCallback(
    async (id: number) => {
      try {
        await invokeCommand("delete_paper", { id });
        await loadPapers();
      } catch (error) {
        console.error("Failed to delete paper:", error);
      }
    },
    [loadPapers],
  );

  const handleRestore = useCallback(
    async (id: number) => {
      try {
        await invokeCommand("restore_paper", { id });
        await loadPapers();
      } catch (error) {
        console.error("Failed to restore paper:", error);
      }
    },
    [loadPapers],
  );

  // Column resizer hooks
  const [draggingColumn, setDraggingColumn] = useState<string | null>(null);
  const [dragStartX, setDragStartX] = useState(0);

  const handleColumnMouseDown = useCallback((e: any, column: string) => {
    e.preventDefault();
    setDraggingColumn(column);
    setDragStartX(e.clientX);
    e.currentTarget.style.cursor = "col-resize";
  }, []);

  const handleColumnMouseMove = useCallback(
    (e: any) => {
      if (!draggingColumn) return;

      const deltaX = e.clientX - dragStartX;
      const containerWidth = window.innerWidth;
      const deltaPercent = (deltaX / containerWidth) * 100;

      setColumnWidths((prev: ColumnWidths) => {
        const newWidths = { ...prev };
        const minWidth = 80;
        const maxWidth = 600;

        const currentWidth = newWidths[draggingColumn as keyof ColumnWidths];
        const newWidth = Math.max(
          minWidth,
          Math.min(maxWidth, currentWidth + deltaPercent),
        );

        newWidths[draggingColumn as keyof ColumnWidths] = newWidth;

        return newWidths;
      });
    },
    [draggingColumn, dragStartX],
  );

  const handleColumnMouseUp = useCallback(() => {
    if (draggingColumn) {
      setDraggingColumn(null);
      document.body.style.cursor = "";
      saveColumnWidths(columnWidths);
    }
    setDragStartX(0);
  }, [draggingColumn, columnWidths, saveColumnWidths]);

  useEffect(() => {
    if (draggingColumn) {
      window.addEventListener("mousemove", handleColumnMouseMove);
      window.addEventListener("mouseup", handleColumnMouseUp);
    } else {
      window.removeEventListener("mousemove", handleColumnMouseMove);
      window.removeEventListener("mouseup", handleColumnMouseUp);
    }

    return () => {
      window.removeEventListener("mousemove", handleColumnMouseMove);
      window.removeEventListener("mouseup", handleColumnMouseUp);
    };
  }, [draggingColumn, handleColumnMouseMove, handleColumnMouseUp]);

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
          overflow: "auto",
          minHeight: 0,
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
                    position: "relative",
                    userSelect: "none",
                    whiteSpace: "nowrap",
                  }}
                  onMouseDown={(e: any) => handleColumnMouseDown(e, "title")}
                >
                  {t("document.title")}
                  <span
                    style={{
                      position: "absolute",
                      right: "0",
                      top: "50%",
                      transform: "translateY(-50%)",
                      color: "gray.4",
                    }}
                  >
                    <IconGripHorizontal size={12} />
                  </span>
                </Table.Th>
                <Table.Th
                  style={{
                    position: "relative",
                    userSelect: "none",
                    whiteSpace: "nowrap",
                  }}
                  onMouseDown={(e: any) => handleColumnMouseDown(e, "authors")}
                >
                  {t("document.authors")}
                  <span
                    style={{
                      position: "absolute",
                      right: "0",
                      top: "50%",
                      transform: "translateY(-50%)",
                      color: "gray.4",
                    }}
                  >
                    <IconGripHorizontal size={12} />
                  </span>
                </Table.Th>
                <Table.Th
                  style={{
                    position: "relative",
                    userSelect: "none",
                    whiteSpace: "nowrap",
                  }}
                  onMouseDown={(e: any) => handleColumnMouseDown(e, "source")}
                >
                  {t("document.source")}
                  <span
                    style={{
                      position: "absolute",
                      right: "0",
                      top: "50%",
                      transform: "translateY(-50%)",
                      color: "gray.4",
                    }}
                  >
                    <IconGripHorizontal size={12} />
                  </span>
                </Table.Th>
                <Table.Th
                  style={{
                    whiteSpace: "nowrap",
                    width: "60px",
                  }}
                >
                  {t("document.year")}
                </Table.Th>
                <Table.Th
                  style={{
                    position: "relative",
                    userSelect: "none",
                    whiteSpace: "nowrap",
                  }}
                  onMouseDown={(e: any) => handleColumnMouseDown(e, "labels")}
                >
                  {t("document.labels")}
                  <span
                    style={{
                      position: "absolute",
                      right: "0",
                      top: "50%",
                      transform: "translateY(-50%)",
                      color: "gray.4",
                    }}
                  >
                    <IconGripHorizontal size={12} />
                  </span>
                </Table.Th>
                <Table.Th style={{ width: 0 }}></Table.Th>
              </Table.Tr>
            </Table.Thead>

            <Table.Tbody>
              {rows.map((record) => {
                const isSelected = selectedDocument?.id === record.id;
                const isExpanded = openedRows.has(record.id);

                return (
                  <React.Fragment key={record.id}>
                    <Table.Tr
                      c={
                        isSelected
                          ? { backgroundColor: `${accentColor}40` }
                          : undefined
                      }
                      style={{ cursor: "pointer", whiteSpace: "nowrap" }}
                      onDoubleClick={() => handleDoubleClick(record)}
                      onClick={() => handleRowClick(record)}
                    >
                      <Table.Td
                        style={{
                          width: `${columnWidths.title}px`,
                          whiteSpace: "nowrap",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                        }}
                      >
                        <Text
                          size="sm"
                          truncate="end"
                          title={record.title}
                          style={{ whiteSpace: "nowrap" }}
                        >
                          {record.title}
                        </Text>
                      </Table.Td>
                      <Table.Td
                        style={{
                          width: `${columnWidths.authors}px`,
                          whiteSpace: "nowrap",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                        }}
                      >
                        <div
                          style={{
                            display: "inline-flex",
                            alignItems: "center",
                            gap: "8px",
                            flexWrap: "nowrap",
                          }}
                        >
                          {record.authors?.slice(0, 3).map((author, index) => (
                            <Badge
                              key={index}
                              variant="filled"
                              size="sm"
                              color={TAG_COLORS.blue}
                              style={{ whiteSpace: "nowrap" }}
                            >
                              {author}
                            </Badge>
                          ))}
                          {record.authors && record.authors.length > 3 && (
                            <Text size="sm" c="dimmed">
                              +{record.authors.length - 3}
                            </Text>
                          )}
                        </div>
                      </Table.Td>
                      <Table.Td
                        style={{
                          width: `${columnWidths.source}px`,
                          whiteSpace: "nowrap",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                        }}
                      >
                        <Text
                          size="sm"
                          truncate="end"
                          title={
                            record.journal_name || record.conference_name || ""
                          }
                          style={{ whiteSpace: "nowrap" }}
                        >
                          {record.journal_name || record.conference_name || ""}
                        </Text>
                      </Table.Td>
                      <Table.Td
                        style={{
                          width: "60px",
                          whiteSpace: "nowrap",
                        }}
                      >
                        <Text size="sm" style={{ whiteSpace: "nowrap" }}>
                          {record.publication_year
                            ? String(record.publication_year)
                            : ""}
                        </Text>
                      </Table.Td>
                      <Table.Td
                        style={{
                          width: `${columnWidths.labels}px`,
                          whiteSpace: "nowrap",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                        }}
                      >
                        <div
                          style={{
                            display: "inline-flex",
                            alignItems: "center",
                            gap: "8px",
                            flexWrap: "nowrap",
                          }}
                        >
                          {record.labels?.slice(0, 3).map((label) => (
                            <Badge
                              key={label.id}
                              variant="filled"
                              size="sm"
                              color={label.color}
                              style={{
                                backgroundColor:
                                  TAG_COLORS[label.color] || TAG_COLORS.blue,
                                color: "#fff",
                                whiteSpace: "nowrap",
                              }}
                            >
                              {label.name}
                            </Badge>
                          ))}
                          {record.labels && record.labels.length > 3 && (
                            <Text size="sm" c="dimmed">
                              +{record.labels.length - 3}
                            </Text>
                          )}
                        </div>
                      </Table.Td>
                      <Table.Td p={0}>
                        {record.attachment_count &&
                          record.attachment_count > 0 && (
                            <ActionIcon
                              size="sm"
                              variant="subtle"
                              onClick={(e: any) => {
                                e.stopPropagation();
                                handleRowExpand(record.id);
                              }}
                              style={{ cursor: "pointer" }}
                            >
                              <IconFile size={16} />
                            </ActionIcon>
                          )}
                      </Table.Td>
                      <Table.Td>
                        <Menu
                          shadow="md"
                          width={200}
                          position="bottom-end"
                          withinPortal
                        >
                          <MenuTarget>
                            <ActionIcon variant="subtle" size="sm">
                              <IconDots size={16} />
                            </ActionIcon>
                          </MenuTarget>

                          <MenuDropdown>
                            {categoryId === "trash" ? (
                              <>
                                <MenuLabel>Actions</MenuLabel>
                                <MenuItem
                                  leftSection={<IconRestore size={14} />}
                                  onClick={() => handleRestore(record.id)}
                                >
                                  {t("dialog.restore")}
                                </MenuItem>
                                <MenuItem
                                  leftSection={<IconTrash size={14} />}
                                  color="red"
                                  onClick={() => handleDelete(record.id)}
                                >
                                  {t("dialog.permanentlyDelete")}
                                </MenuItem>
                              </>
                            ) : (
                              <>
                                <MenuLabel>Actions</MenuLabel>
                                <MenuItem
                                  leftSection={<IconPaperclip size={14} />}
                                  onClick={async () => {
                                    const selected = await open({
                                      multiple: false,
                                      directory: false,
                                    });
                                    if (selected) {
                                      const filePath = Array.isArray(selected)
                                        ? selected[0]
                                        : selected;
                                      if (filePath) {
                                        await invokeCommand("add_attachment", {
                                          paperId: record.id,
                                          filePath: filePath,
                                        });
                                        window.dispatchEvent(
                                          new CustomEvent(
                                            "attachment-updated",
                                            {
                                              detail: { paperId: record.id },
                                            },
                                          ),
                                        );
                                        await loadPapers();
                                      }
                                    }
                                  }}
                                >
                                  添加附件
                                </MenuItem>
                                <MenuItem
                                  leftSection={<IconFolderOpen size={14} />}
                                  onClick={async () => {
                                    await invokeCommand("open_paper_folder", {
                                      paperId: record.id,
                                    });
                                  }}
                                >
                                  打开附件文件夹
                                </MenuItem>
                                <MenuDivider />
                                <MenuItem
                                  leftSection={<IconTrash size={14} />}
                                  color="red"
                                  onClick={() => handleDelete(record.id)}
                                >
                                  {t("dialog.delete")}
                                </MenuItem>
                              </>
                            )}
                          </MenuDropdown>
                        </Menu>
                      </Table.Td>
                    </Table.Tr>

                    {isExpanded && (
                      <Table.Tr>
                        <Table.Td colSpan={7} style={{ padding: 0 }}>
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
