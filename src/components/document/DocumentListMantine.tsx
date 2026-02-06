import React, { useState, useEffect, useCallback } from "react";
import { Table, Text, Badge, Group, Stack, rem, Tooltip } from "@mantine/core";
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

const TABLE_FONT_SIZE = 12;

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
            <Text size="xs" c="dimmed">
              -
            </Text>
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
                    width: "40px",
                    textAlign: "center",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                ></Table.Th>
                <Table.Th
                  style={{
                    width: "1",
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
                    width: "250px",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                >
                  {t("document.source")}
                </Table.Th>
                <Table.Th
                  style={{
                    width: "25px",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                >
                  {t("document.year")}
                </Table.Th>
                <Table.Th
                  style={{
                    width: "50px",
                    fontSize: `${TABLE_FONT_SIZE}px`,
                    verticalAlign: "middle",
                  }}
                >
                  {t("document.labels")}
                </Table.Th>
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
                        {record.attachment_count &&
                          record.attachment_count > 0 && (
                            <Text
                              size="sm"
                              style={{
                                cursor: "pointer",
                                fontSize: "14px",
                                fontWeight: "bold",
                                color: "var(--mantine-color-gray-6)",
                              }}
                            >
                              {">"}
                            </Text>
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
                          style={{ whiteSpace: "nowrap" }}
                        >
                          {record.title}
                        </Text>
                      </Table.Td>
                      <Table.Td
                        style={{
                          width: "100px",
                          verticalAlign: "middle",
                          paddingLeft: 8,
                          paddingRight: 8,
                        }}
                      >
                        <Tooltip.Floating
                          label={
                            <div
                              style={{
                                display: "inline-flex",
                                alignItems: "center",
                                gap: "8px",
                                flexWrap: "wrap",
                                maxWidth: 300,
                              }}
                            >
                              {record.authors?.map((author, index) => (
                                <Tag key={index} color="blue">
                                  {author}
                                </Tag>
                              ))}
                            </div>
                          }
                          position="top"
                          withinPortal={false}
                        >
                          <div
                            style={{
                              display: "inline-flex",
                              alignItems: "center",
                              gap: "4px",
                              flexWrap: "nowrap",
                              height: "100%",
                            }}
                          >
                            {record.authors
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
                                      {record.authors.length - 1}
                                    </Badge>
                                  )}
                                </Tag>
                              ))}
                          </div>
                        </Tooltip.Floating>
                      </Table.Td>
                      <Table.Td
                        style={{
                          width: "250px",
                          verticalAlign: "middle",
                          paddingLeft: 8,
                          paddingRight: 8,
                        }}
                      >
                        <Text
                          size="sm"
                          truncate="end"
                          title={
                            record.journal_name || record.conference_name || ""
                          }
                          style={{
                            whiteSpace: "nowrap",
                            fontSize: `${TABLE_FONT_SIZE}px`,
                            lineHeight: 1,
                            margin: 0,
                            padding: 0,
                            display: "block",
                          }}
                        >
                          {record.journal_name || record.conference_name || ""}
                        </Text>
                      </Table.Td>
                      <Table.Td
                        style={{
                          width: "25px",
                          verticalAlign: "middle",
                          paddingLeft: 8,
                          paddingRight: 8,
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
                            display: "block",
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
                          paddingLeft: 8,
                          paddingRight: 8,
                        }}
                      >
                        <div
                          style={{
                            display: "inline-flex",
                            alignItems: "center",
                            gap: "4px",
                            flexWrap: "nowrap",
                            height: "100%",
                          }}
                        >
                          {record.labels?.slice(0, 3).map((label) => (
                            <Tag
                              key={label.id}
                              color={TAG_COLORS[label.color] || TAG_COLORS.blue}
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
                          ))}
                          {record.labels && record.labels.length > 3 && (
                            <Text
                              size="sm"
                              c="dimmed"
                              style={{
                                fontSize: `${TABLE_FONT_SIZE}px`,
                                margin: 0,
                              }}
                            >
                              +{record.labels.length - 3}
                            </Text>
                          )}
                        </div>
                      </Table.Td>
                    </Table.Tr>

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
