import { truncateMiddle } from "../../utils/capitalize";
import SearchableTable from "../../components/tables/SearchableTable";
import getAccessTokens from "../queries/getAccessTokens";
import deleteAccessToken from "../mutations/deleteAccessToken";
import { ClipboardCopyIcon, PlusCircleIcon } from "@heroicons/react/outline";
import copy from "copy-to-clipboard";
import { useState } from "react";
import { useModal } from "../../contexts/modal";
import { TrashIcon } from "@heroicons/react/outline";
import { useConfirm } from "../../contexts/confirm";
import { useQueryClient } from "react-query";
import { Link } from "react-router-dom";
import { AccessToken } from "@l2-technology/sensei-client";
import formatDistanceToNow from "date-fns/formatDistanceToNow";

const SimpleColumn = ({ value, className }) => {
  return (
    <td
      className={`px-6 py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value}
    </td>
  );
};

const ActionsColumn = ({ value, token, className }) => {
  const { showConfirm } = useConfirm();
  const queryClient = useQueryClient();

  const deleteTokenClicked = () => {
    showConfirm({
      title: "Are you sure you want to delete this token?",
      description:
        "A deleted token can no longer be used to make authenticated requests",
      ctaText: "Yes, delete it",
      callback: async () => {
        await deleteAccessToken(token.id);
        queryClient.invalidateQueries("tokens");
      },
    });
  };

  return (
    <td
      className={`px-6 py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      <TrashIcon
        className="inline-block h-6 cursor-pointer"
        onClick={deleteTokenClicked}
      />
    </td>
  );
};

const StatusColumn = ({ token, value, className }) => {
  const expiresAt = parseInt(token.expiresAt, 10);
  const now = new Date().getTime();
  const expired = expiresAt > 0 && expiresAt < now;

  return (
    <td
      className={`px-6 py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {expired && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-200 text-red-800">
          Expired
        </span>
      )}
      {!expired && expiresAt === 0 && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
          Active
        </span>
      )}
      {!expired && expiresAt > 0 && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
          Expires in {formatDistanceToNow(expiresAt)}
        </span>
      )}
    </td>
  );
};

const SingleUseColumn = ({ value, className }) => {
  return (
    <td
      className={`px-6 py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value && <span className="">Single</span>}
      {!value && <span className="">Unlimited</span>}
    </td>
  );
};

const TokenColumn = ({ token, value, className }) => {
  let [copied, setCopied] = useState(false);

  const copyClicked = () => {
    copy(token.token);
    setCopied(true);
    setTimeout(() => {
      setCopied(false);
    }, 1000);
  };

  return copied ? (
    <td
      className={`px-6 py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
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
      className={`group cursor-pointer px-6 py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {truncateMiddle(value, 10)}{" "}
      <span className="inline-block group-hover:hidden">
        &nbsp;&nbsp;&nbsp;&nbsp;
      </span>
      <ClipboardCopyIcon className="w-4 text-gray-500 hidden group-hover:inline-block" />
    </td>
  );
};

const TokenRow = ({ result, extraClass, attributes }) => {
  let columnKeyComponentMap = {
    singleUse: SingleUseColumn,
    token: TokenColumn,
    actions: ActionsColumn,
    status: StatusColumn,
  };

  return (
    <tr className={`border-b border-plum-200 ${extraClass}`}>
      {attributes.map(({ key, label, className }) => {
        let value = result[key];
        let ColumnComponent = columnKeyComponentMap[key]
          ? columnKeyComponentMap[key]
          : SimpleColumn;

        return (
          <ColumnComponent
            key={key}
            token={result}
            value={value}
            className={className}
          />
        );
      })}
    </tr>
  );
};

const TokensListCard = () => {
  const emptyTableHeadline = "No tokens found";
  const emptyTableSubtext = "Try changing the search term";
  const searchBarPlaceholder = "Search";

  const attributes = [
    {
      key: "name",
      label: "Name",
    },
    {
      key: "token",
      label: "Token",
    },
    {
      key: "scope",
      label: "Scope",
    },
    {
      key: "singleUse",
      label: "Usage Limit",
    },
    {
      key: "status",
      label: "Status",
    },
    {
      key: "actions",
      label: "Actions",
    },
  ];

  const transformResults = (tokens: AccessToken[]) => {
    return tokens.map((token) => {
      return {
        ...token,
        actions: "Action",
        status: "Status",
      };
    });
  };

  const queryFunction = async ({ queryKey }) => {
    const [_key, { page, searchTerm, take }] = queryKey;
    const response = await getAccessTokens({ page, searchTerm, take });
    return {
      results: transformResults(response.tokens),
      hasMore: response.pagination.hasMore,
      total: response.pagination.total,
    };
  };

  return (
    <SearchableTable
      attributes={attributes}
      queryKey="tokens"
      queryFunction={queryFunction}
      emptyTableHeadline={emptyTableHeadline}
      emptyTableSubtext={emptyTableSubtext}
      searchBarPlaceholder={searchBarPlaceholder}
      hasHeader
      itemsPerPage={5}
      RowComponent={TokenRow}
    />
  );
};

export default TokensListCard;
