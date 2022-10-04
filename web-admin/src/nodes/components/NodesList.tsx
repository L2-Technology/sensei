import { truncateMiddle } from "../../utils/capitalize";
import SearchableTable from "../../components/tables/SearchableTable";
import getNodes from "../queries/getNodes";
import { ClipboardCopyIcon, PlusCircleIcon } from "@heroicons/react/outline";
import copy from "copy-to-clipboard";
import { useState } from "react";
import StartNodeForm from "../components/StartNodeForm";
import { useModal } from "../../contexts/modal";
import {
  PlayIcon,
  StopIcon,
  DotsHorizontalIcon,
} from "@heroicons/react/outline";
import { useConfirm } from "../../contexts/confirm";
import adminStopNode from "../mutations/adminStopNode";
import { useQueryClient } from "react-query";
import { Node } from "@l2-technology/sensei-client";
import Dropdown from "src/components/layout/app/Dropdown";

const SimpleColumn = ({ value, className }) => {
  return (
    <td
      className={`p-3 md:px-6 md:py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value}
    </td>
  );
};

const RoleColumn = ({ value, className }) => {
  const displayRole = value === 0 ? "Root" : "Default";
  return (
    <td
      className={`p-3 md:px-6 md:py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {displayRole}
    </td>
  );
};

const ActionsColumn = ({ node, className }) => {
  const { showModal, hideModal } = useModal();
  const { showConfirm } = useConfirm();
  const queryClient = useQueryClient();

  const nodeStarted = () => {
    queryClient.invalidateQueries("nodes");
    hideModal();
  };

  const startNodeClicked = async () => {
    showModal({
      component: <StartNodeForm pubkey={node.id} callback={nodeStarted} />,
    });
  };

  const stopNodeClicked = () => {
    showConfirm({
      title: "Are you sure you want to stop this node?",
      description:
        "A stopped node can no longer send, receive, or route payments.  The node will also no longer be monitoring the chain for misbehavior.",
      ctaText: "Yes, stop it",
      callback: async () => {
        await adminStopNode(node.id);
        queryClient.invalidateQueries("nodes");
      },
    });
  };

  const actionItems = [
    {
      label: node.status === 0 ? "start" : "stop",
      icon:
        node.status === 0 ? (
          <PlayIcon className="w-6" />
        ) : (
          <StopIcon className="w-6" />
        ),
      onClick: node.status === 0 ? startNodeClicked : stopNodeClicked,
      className: node.status === 0 ? "text-green-400" : "text-yellow-400",
    },
  ];

  return (
    <td
      className={`p-3 md:px-6 md:py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      <Dropdown
        items={actionItems}
        button={<DotsHorizontalIcon className="w-6" />}
      />
    </td>
  );
};

const StatusColumn = ({ value, className }) => {
  let dot = "bg-white";
  let displayValue = "Stopped";

  if (value === 1) {
    dot = "bg-gradient-to-br from-green-400 to-green-700";
    displayValue = "Running";
  }

  if (value === 0) dot = "bg-gradient-to-br from-yellow-400 to-yellow-700";

  return (
    <td
      className={`p-3 md:px-6 md:py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      <div className="flex items-center justify-center md:justify-start">
        <div className={`${dot} mr-2 h-4 w-4 rounded-full shadow-md`} />
        <span className="capitalize hidden md:block">{displayValue}</span>
      </div>
    </td>
  );
};

const ConnectionInfoColumn = ({ node, value, className }) => {
  let [copied, setCopied] = useState(false);

  const copyClicked = () => {
    copy(`${node.id}@${node.listenAddr}:${node.listenPort}`);
    setCopied(true);
    setTimeout(() => {
      setCopied(false);
    }, 1000);
  };

  return copied ? (
    <td
      className={`p-3 md:px-6 md:py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      Copied! &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
      &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
      &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
      &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
      &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
      &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    </td>
  ) : (
    <td
      onClick={copyClicked}
      className={`group cursor-pointer p-3 md:px-6 md:py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value}{" "}
      <span className="inline-block group-hover:hidden">
        &nbsp;&nbsp;&nbsp;&nbsp;
      </span>
      <ClipboardCopyIcon className="w-4 text-gray-500 hidden group-hover:inline-block" />
    </td>
  );
};

const NodeRow = ({ result, extraClass, attributes }) => {
  let columnKeyComponentMap = {
    status: StatusColumn,
    connectionInfo: ConnectionInfoColumn,
    actions: ActionsColumn,
    role: RoleColumn,
  };

  return (
    <tr className={`${extraClass}`}>
      {attributes.map(({ key, className }) => {
        let value = result[key];
        let ColumnComponent = columnKeyComponentMap[key]
          ? columnKeyComponentMap[key]
          : SimpleColumn;

        return (
          <ColumnComponent
            key={key}
            node={result}
            value={value}
            className={className}
          />
        );
      })}
    </tr>
  );
};

const NodesListCard = () => {
  const emptyTableHeadline = "No nodes found";
  const emptyTableSubtext = "Try changing the search term";
  const searchBarPlaceholder = "Search";

  const attributes = [
    {
      key: "username",
      label: "Username",
    },
    {
      key: "alias",
      label: "Alias",
    },
    {
      key: "role",
      label: "Role",
    },
    {
      key: "connectionInfo",
      label: "Connection Info",
    },
    {
      key: "status",
      label: "Status",
    },
    {
      key: "actions",
      label: "Actions",
      className: "text-center",
    },
  ];

  const transformResults = (nodes: Node[]) => {
    return nodes.map((node) => {
      return {
        ...node,
        connectionInfo: `${truncateMiddle(node.id, 10)}@${
          node.listenAddr
        }:${node.listenPort}`,
        actions: "Action",
      };
    });
  };

  const queryFunction = async ({ queryKey }) => {
    const [_key, { page, searchTerm, take }] = queryKey;
    const response = await getNodes({ page, searchTerm, take });
    return {
      results: transformResults(response.nodes),
      hasMore: response.pagination.hasMore,
      total: response.pagination.total,
    };
  };

  return (
    <SearchableTable
      attributes={attributes}
      queryKey="nodes"
      queryFunction={queryFunction}
      emptyTableHeadline={emptyTableHeadline}
      emptyTableSubtext={emptyTableSubtext}
      searchBarPlaceholder={searchBarPlaceholder}
      hasHeader
      itemsPerPage={5}
      RowComponent={NodeRow}
      striped={true}
    />
  );
};

export default NodesListCard;
