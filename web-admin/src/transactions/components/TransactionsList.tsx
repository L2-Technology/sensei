import getTransactions from "../queries/getTransactions";
import SearchableTable from "../../components/tables/SearchableTable";
import { truncateMiddle } from "../../utils/capitalize";
import { TransactionDetails } from "@l2-technology/sensei-client";

const StatusColumn = ({ value, className }) => {
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value === "unconfirmed" && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
          Unconfirmed
        </span>
      )}

      {value === "confirmed" && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
          Confirmed
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
      {new Intl.NumberFormat().format(value)}
    </td>
  );
};

const TransactionRow = ({ result, extraClass, attributes }) => {
  let columnKeyComponentMap = {
    amount: AmountColumn,
    fee: AmountColumn,
    status: StatusColumn,
  };

  return (
    <tr className={`border-b border-plum-200 ${extraClass}`}>
      {attributes.map(({ key, className }) => {
        let value = result[key];
        let ColumnComponent = columnKeyComponentMap[key]
          ? columnKeyComponentMap[key]
          : SimpleColumn;
        return (
          <ColumnComponent
            key={key}
            transaction={result}
            value={value}
            className={className}
          />
        );
      })}
    </tr>
  );
};

const TransactionsList = () => {
  const emptyTableHeadline = "No transactions found";
  const emptyTableSubtext = "Try changing the search term";
  const searchBarPlaceholder = "Search";

  const attributes = [
    {
      key: "displayTransactionId",
      label: "Transaction Id",
    },
    {
      key: "amount",
      label: "Amount (sats)",
    },
    {
      key: "fee",
      label: "Fee (sats)",
    },
    {
      key: "status",
      label: "Status",
    },
  ];

  const transformResults = (transactions: TransactionDetails[]) => {
    return transactions.map((transaction) => {
      let status = transaction.confirmationTime ? `confirmed` : "unconfirmed";

      return {
        ...transaction,
        id: transaction.txid,
        amount: transaction.received - transaction.sent,
        status,
        displayTransactionId: truncateMiddle(transaction.txid, 10),
      };
    });
  };

  const queryFunction = async ({ queryKey }) => {
    const [_key, { page, searchTerm, take }] = queryKey;
    const { transactions, pagination } = await getTransactions({
      page,
      searchTerm,
      take,
    });
    return {
      results: transformResults(transactions),
      hasMore: pagination.hasMore,
      total: pagination.total,
    };
  };

  return (
    <SearchableTable
      attributes={attributes}
      queryKey="transactions"
      queryFunction={queryFunction}
      emptyTableHeadline={emptyTableHeadline}
      emptyTableSubtext={emptyTableSubtext}
      searchBarPlaceholder={searchBarPlaceholder}
      hasHeader
      itemsPerPage={5}
      RowComponent={TransactionRow}
    />
  );
};

export default TransactionsList;
