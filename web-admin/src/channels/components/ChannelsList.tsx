import getChannels from "../queries/getChannels";
import { useQueryClient } from "react-query";
import {
  EyeOffIcon,
  SpeakerphoneIcon,
  StopIcon,
  TrashIcon,
} from "@heroicons/react/outline";
import SearchableTable from "../../components/tables/SearchableTable";
import { truncateMiddle } from "../../utils/capitalize";
import { useConfirm } from "../../contexts/confirm";
import closeChannel from "../mutations/closeChannel";
import { Channel } from "@l2-technology/sensei-client";

const StatusColumn = ({ value, className }) => {
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value === "pending_confirmations" && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
          Pending Confirmations
        </span>
      )}
      {value === "counterparty_offline" && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-300 text-gray-600">
          Counterparty Offline
        </span>
      )}
      {value === "ready" && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
          Ready
        </span>
      )}
    </td>
  );
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

const AmountColumn = ({ value, className }) => {
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {new Intl.NumberFormat().format(value / 1000)}
    </td>
  );
};

const VisibilityColumn = ({ value, className }) => {
  let Icon = value ? SpeakerphoneIcon : EyeOffIcon;
  let displayValue = value ? "Public" : "Private";
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      <Icon className="w-4 inline-block" /> {displayValue}
    </td>
  );
};

const ActionsColumn = ({ channel, className }) => {
  const { showConfirm } = useConfirm();
  const queryClient = useQueryClient();

  const closeChannelClicked = () => {
    showConfirm({
      title: "Are you sure you want to close this channel?",
      description:
        "A closed channel can no longer be used to send or receive payments.",
      ctaText: "Yes, close it",
      callback: async () => {
        await closeChannel(channel.channelId, false);
        queryClient.invalidateQueries("channels");
      },
    });
  };

  const forceCloseChannelClicked = () => {
    showConfirm({
      title: "Are you sure you want to force close this channel?",
      description:
        "A closed channel can no longer be used to send or receive payments. Force closing a channel will cost you extra in fees.  You should really prefer to cooperatively close this channel if possible.",
      ctaText: "Yes, force close it",
      callback: async () => {
        await closeChannel(channel.channelId, true);
        queryClient.invalidateQueries("channels");
      },
    });
  };

  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      <StopIcon
        className="inline-block h-6 cursor-pointer"
        onClick={closeChannelClicked}
      />
      <TrashIcon
        className="inline-block h-6 cursor-pointer"
        onClick={forceCloseChannelClicked}
      />
    </td>
  );
};

const AliasColumn = ({ value, className }) => {
  let displayValue = value || "Unknown";
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {displayValue}
    </td>
  );
};

const ChannelRow = ({ result, extraClass, attributes }) => {
  let columnKeyComponentMap = {
    inboundCapacityMsat: AmountColumn,
    outboundCapacityMsat: AmountColumn,
    balanceMsat: AmountColumn,
    isPublic: VisibilityColumn,
    status: StatusColumn,
    alias: AliasColumn,
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
            channel={result}
            value={value}
            className={className}
          />
        );
      })}
    </tr>
  );
};

const ChannelsList = () => {
  const emptyTableHeadline = "No channels found";
  const emptyTableSubtext = "Try changing the search term";
  const searchBarPlaceholder = "Search";

  const attributes = [
    {
      key: "displayChannelId",
      label: "Channel Id",
    },
    {
      key: "balanceMsat",
      label: "Balance (sats)",
    },
    {
      key: "inboundCapacityMsat",
      label: "Receive Capacity (sats)",
    },
    {
      key: "outboundCapacityMsat",
      label: "Send Capacity (sats)",
    },
    {
      key: "status",
      label: "Status",
    },
    {
      key: "isPublic",
      label: "Visibility",
    },
    {
      key: "displayCounterparty",
      label: "Counterparty",
    },
    {
      key: "actions",
      label: "Actions",
    },
  ];

  const transformResults = (channels: Channel[]) => {
    return channels.map((channel) => {
      let status = channel.isChannelReady
        ? channel.isUsable
          ? "ready"
          : "counterparty_offline"
        : "pending_confirmations";

      return {
        ...channel,
        id: channel.channelId,
        status,
        actions: "",
        displayChannelId: truncateMiddle(channel.channelId, 10),
        displayCounterparty:
          channel.alias || truncateMiddle(channel.counterpartyPubkey, 10),
      };
    });
  };

  const queryFunction = async ({ queryKey }) => {
    const [_key, { page, searchTerm, take }] = queryKey;
    const { channels, pagination } = await getChannels({
      page,
      searchTerm,
      take,
    });
    return {
      results: transformResults(channels),
      hasMore: pagination.hasMore,
      total: pagination.total,
    };
  };

  // TODO: when channels are stored in db and can be in error state
  //       need to make sure we aren't polling failed/closed channels
  const refetchInterval = (data, query) => {
    const hasPendingChannel = data?.results.find(channel => {
      return channel.status !== "ready"
    })
    return hasPendingChannel ? 1000 : false
  }

  return (
    <SearchableTable
      attributes={attributes}
      queryKey="channels"
      queryFunction={queryFunction}
      emptyTableHeadline={emptyTableHeadline}
      emptyTableSubtext={emptyTableSubtext}
      searchBarPlaceholder={searchBarPlaceholder}
      hasHeader
      itemsPerPage={5}
      RowComponent={ChannelRow}
      striped={true}
      refetchInterval={refetchInterval}
    />
  );
};

export default ChannelsList;
