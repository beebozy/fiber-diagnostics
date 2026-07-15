"use client";

import { useEffect, useState } from "react";
import { fetchNodes, addNode, deleteNode } from "@/lib/api";
import { MonitoredNode } from "@/lib/types";
import Header from "@/components/Header";
import styles from "./page.module.css";

function IconTrash() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
      <line x1="10" y1="11" x2="10" y2="17" />
      <line x1="14" y1="11" x2="14" y2="17" />
    </svg>
  );
}

function IconPlus() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  );
}

export default function NodesPage() {
  const [nodes, setNodes] = useState<MonitoredNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Form states
  const [nodeId, setNodeId] = useState("");
  const [nodeName, setNodeName] = useState("");
  const [rpcUrl, setRpcUrl] = useState("");
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function loadNodes() {
    try {
      setLoading(true);
      const data = await fetchNodes();
      setNodes(data);
      setError(null);
    } catch (err: any) {
      setError(err.message || "Failed to load monitored nodes");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadNodes();
  }, []);

  async function handleAddNode(e: React.FormEvent) {
    e.preventDefault();
    if (!nodeId.trim() || !nodeName.trim() || !rpcUrl.trim()) {
      setFormError("All fields are required.");
      return;
    }

    try {
      setSubmitting(true);
      setFormError(null);
      await addNode(nodeId.trim(), nodeName.trim(), rpcUrl.trim());
      
      // Clear form
      setNodeId("");
      setNodeName("");
      setRpcUrl("");
      
      // Reload list
      await loadNodes();
    } catch (err: any) {
      setFormError(err.message || "Failed to add node. Check if URL is unique.");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleDelete(id: string) {
    if (!confirm(`Are you sure you want to remove node ${id}?`)) {
      return;
    }

    try {
      await deleteNode(id);
      await loadNodes();
    } catch (err: any) {
      alert(err.message || "Failed to delete node.");
    }
  }

  return (
    <div className={styles.wrapper}>
      <Header title="Monitored Nodes Configuration" />

      <div className={styles.container}>
        {/* Left column: Monitored Nodes List */}
        <div className={styles.listSection}>
          <h3 className={styles.sectionTitle}>Currently Monitored Nodes</h3>
          
          {loading && nodes.length === 0 ? (
            <div className={styles.loadingState}>
              <div className={styles.shimmerRow} />
              <div className={styles.shimmerRow} />
            </div>
          ) : error ? (
            <div className={styles.errorBanner}>{error}</div>
          ) : nodes.length === 0 ? (
            <div className={styles.emptyState}>
              No nodes registered. Use the configuration form to add your first node.
            </div>
          ) : (
            <div className={styles.nodeList}>
              {nodes.map((node) => (
                <div key={node.id} className={styles.nodeCard}>
                  <div className={styles.nodeDetails}>
                    <div className={styles.nodeHeader}>
                      <span className={styles.nodeName}>{node.name}</span>
                      <code className={styles.nodeId}>{node.id}</code>
                    </div>
                    <div className={styles.nodeUrl}>{node.rpc_url}</div>
                  </div>
                  <button
                    onClick={() => handleDelete(node.id)}
                    className={styles.deleteButton}
                    title="Remove node"
                  >
                    <IconTrash />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Right column: Form to Add Nodes */}
        <div className={styles.formSection}>
          <h3 className={styles.sectionTitle}>Register New Node</h3>
          
          <form onSubmit={handleAddNode} className={styles.form}>
            {formError && <div className={styles.formErrorBanner}>{formError}</div>}
            
            <div className={styles.field}>
              <label className={styles.label} htmlFor="nodeId">Node ID / Alias</label>
              <input
                id="nodeId"
                type="text"
                placeholder="e.g. node3"
                value={nodeId}
                onChange={(e) => setNodeId(e.target.value)}
                className={styles.input}
                disabled={submitting}
                required
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="nodeName">Display Name</label>
              <input
                id="nodeName"
                type="text"
                placeholder="e.g. Local Node C"
                value={nodeName}
                onChange={(e) => setNodeName(e.target.value)}
                className={styles.input}
                disabled={submitting}
                required
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="rpcUrl">JSON-RPC URL</label>
              <input
                id="rpcUrl"
                type="url"
                placeholder="e.g. http://127.0.0.1:8247"
                value={rpcUrl}
                onChange={(e) => setRpcUrl(e.target.value)}
                className={styles.input}
                disabled={submitting}
                required
              />
            </div>

            <button
              type="submit"
              className={styles.submitButton}
              disabled={submitting}
            >
              <span className={styles.plusIcon}><IconPlus /></span>
              {submitting ? "Registering..." : "Add Monitored Node"}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}
