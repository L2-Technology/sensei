import getKnownPeers from "../queries/getKnownPeers";
import { useQueryClient } from "react-query";
import {
  TrashIcon,
} from "@heroicons/react/outline";
import SearchableTable from "../../components/tables/SearchableTable";
import { useConfirm } from "../../contexts/confirm";
import removeKnownPeer from "../mutations/removeKnownPeer";
import addKnownPeer from "../mutations/addKnownPeer";
import { KnownPeer } from "@l2-technology/sensei-client";
import EditableLabelColumn from "src/components/tables/EditableLabelColumn";

const LabelColumn = ({ knownPeer, value }) => {
  
  const updateLabel = async (newLabel) => {
    await addKnownPeer(knownPeer.pubkey, newLabel, knownPeer.zeroConf);
  }

  return <EditableLabelColumn 
    label={value} 
    updateLabel={updateLabel} 
    queryKey={"knownPeers"} 
  />
  
};

const SimpleColumn = ({ value, className }) => {
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value}
    </td>
  );
};

const ZeroConfColumn = ({ knownPeer, value, className }) => {
  let queryClient = useQueryClient();
  const toggleZeroConf = async () => {
    await addKnownPeer(knownPeer.pubkey, knownPeer.label, !knownPeer.zeroConf)
    queryClient.invalidateQueries("knownPeers")
  }

  return (
    <td
      onClick={toggleZeroConf}
      className={`p-3 md:px-6 md:py-4 select-none whitespace-nowrap text-sm leading-5 font-medium text-light-plum cursor-pointer ${className}`}
    >
      {value && (<div className="">Accept 0-Conf</div>)}
      {!value && (<div className="">Requires Confirmations</div>)}
    </td>
  );
};

const ActionsColumn = ({ knownPeer, className }) => {
  const { showConfirm } = useConfirm();
  const queryClient = useQueryClient();

  let removePeerMessage = "You cannot undo this action without manually adding the peer again."
  
  if(knownPeer.zeroConf) {
    removePeerMessage += " You will no longer automatically accept 0-conf channels from this peer."
  }

  const removeKnownPeerClicked = () => {
    showConfirm({
      title: "Are you sure you want to remove this known peer?",
      description: removePeerMessage,
      ctaText: "Yes, remove them",
      callback: async () => {
        await removeKnownPeer(knownPeer.pubkey);
        queryClient.invalidateQueries("knownPeers");
      },
    });
  };
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      <TrashIcon
        className="inline-block h-6 cursor-pointer"
        onClick={removeKnownPeerClicked}
      />
    </td>
  );
};

const KnownPeerRow = ({ result, extraClass, attributes }) => {
  let columnKeyComponentMap = {
    label: LabelColumn,
    zeroConf: ZeroConfColumn,
    actions: ActionsColumn,
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
            knownPeer={result}
            value={value}
            className={className}
          />
        );
      })}
    </tr>
  );
};

const KnownPeersList = () => {
  const emptyTableHeadline = "No peers found";
  const emptyTableSubtext = "Try changing the search term";
  const searchBarPlaceholder = "Search";

  const attributes = [
    {
      key: "label",
      label: "Label",
    },
    {
      key: "pubkey",
      label: "Pubkey",
    },
    {
      key: "zeroConf",
      label: "Inbound Channel Requests",
    },
    {
      key: "actions",
      label: "Actions",
    },
  ];

  const transformResults = (knownPeers: KnownPeer[]) => {
    return knownPeers.map((knownPeer) => {
      return {
        ...knownPeer,
        id: knownPeer.pubkey,
        actions: "",
      };
    });
  };

  const queryFunction = async ({ queryKey }) => {
    const [_key, { page, searchTerm, take }] = queryKey;
    const { peers, pagination } = await getKnownPeers({
      page,
      searchTerm,
      take,
    });
    return {
      results: transformResults(peers),
      hasMore: pagination.hasMore,
      total: pagination.total,
    };
  };

  return (
    <SearchableTable
      attributes={attributes}
      queryKey="knownPeers"
      queryFunction={queryFunction}
      emptyTableHeadline={emptyTableHeadline}
      emptyTableSubtext={emptyTableSubtext}
      searchBarPlaceholder={searchBarPlaceholder}
      hasHeader
      itemsPerPage={5}
      RowComponent={KnownPeerRow}
      striped={true}
    />
  );
};

export default KnownPeersList;
