import { useState } from "react";
import { useAppStore, FileNode } from "../stores/appStore";
import { ChevronRight, ChevronDown, File, Folder } from "lucide-react";

interface FileTreeProps {
  nodes: FileNode[];
}

function FileTreeNode({ node, depth = 0 }: { node: FileNode; depth?: number }) {
  const [expanded, setExpanded] = useState(depth === 0);
  const { selectFile, selectedFilePath } = useAppStore();

  const handleClick = async () => {
    if (node.is_directory) {
      setExpanded(!expanded);
    } else {
      selectFile(node.path);
    }
  };

  const isSelected = selectedFilePath === node.path;

  return (
    <div>
      <div
        className={`flex items-center gap-1 py-1 px-2 cursor-pointer hover:bg-accent rounded ${
          isSelected ? "bg-accent" : ""
        }`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={handleClick}
      >
        {node.is_directory ? (
          <>
            {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
            <Folder size={14} className="text-blue-500" />
          </>
        ) : (
          <>
            <span className="w-4" />
            <File size={14} className="text-gray-500" />
          </>
        )}
        <span className="text-sm truncate">{node.name}</span>
        {node.size && (
          <span className="text-xs text-muted-foreground ml-auto">
            {(node.size / 1024).toFixed(1)}KB
          </span>
        )}
      </div>
    </div>
  );
}

export function FileTree({ nodes }: FileTreeProps) {
  if (nodes.length === 0) {
    return <p className="text-sm text-muted-foreground p-2">No files</p>;
  }

  return (
    <div className="font-mono text-sm">
      {nodes.map((node) => (
        <FileTreeNode key={node.path} node={node} />
      ))}
    </div>
  );
}