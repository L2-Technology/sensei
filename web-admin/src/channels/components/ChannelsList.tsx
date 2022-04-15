import getChannels from "../queries/getChannels";
import { useQueryClient } from "react-query";
import {
  EyeOffIcon,
  PlusIcon,
  SpeakerphoneIcon,
  StopIcon,
  TrashIcon,
} from "@heroicons/react/outline";
import { Link } from "react-router-dom";
import SearchableTable from "../../components/tables/SearchableTable";
import { truncateMiddle } from "../../utils/capitalize";
import { useConfirm } from "../../contexts/confirm";
import closeChannel from "../mutations/closeChannel";
import { Channel } from "@l2-technology/sensei-client";

const NoChannels = () => {
  return (
    <div className="text-center">
      <svg
        className="mx-auto h-12 w-12 text-gray-400"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        aria-hidden="true"
      >
        <path
          vectorEffect="non-scaling-stroke"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"
        />
      </svg>
      <h3 className="mt-2 text-sm font-medium text-light-plum">No channels</h3>
      <p className="mt-1 text-sm text-gray-500">
        Open a channel to start sending and receiving over lightning.
      </p>
      <div className="mt-6">
        <Link
          to="/admin/channels/open"
          className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
        >
          <PlusIcon className="-ml-1 mr-2 h-5 w-5" aria-hidden="true" />
          Open a Channel
        </Link>
      </div>
    </div>
  );
};

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

const SimpleColumn = ({ channel, value, className }) => {
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value}
    </td>
  );
};

const AmountColumn = ({ channel, value, className }) => {
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {new Intl.NumberFormat().format(value / 1000)}
    </td>
  );
};

const VisibilityColumn = ({ channel, value, className }) => {
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

const ActionsColumn = ({ value, channel, className }) => {
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

const AliasColumn = ({ channel, value, className }) => {
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
    isPublic: VisibilityColumn,
    status: StatusColumn,
    alias: AliasColumn,
    actions: ActionsColumn,
  };

  return (
    <tr className={`${extraClass}`}>
      {attributes.map(({ key, label, className }) => {
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
      let status = channel.isFundingLocked
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
    const {channels, pagination } = await getChannels({
      page,
      searchTerm,
      take,
    });
    return {
      results: transformResults(channels),
      hasMore: pagination.hasMore,
      total: pagination.total
    };
  };

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
    />
  );
};

export default ChannelsList;
