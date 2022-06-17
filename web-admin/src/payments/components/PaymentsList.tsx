import { truncateMiddle } from "../../utils/capitalize";
import SearchableTable from "../../components/tables/SearchableTable";
import getPayments from "../queries/getPayments";
import labelPayment from "../mutations/labelPayment";
import copy from "copy-to-clipboard";
import { Payment } from "@l2-technology/sensei-client";
import { useState } from "react";
import {
  ClipboardCopyIcon,
} from "@heroicons/react/outline";
import EditableLabelColumn from "src/components/tables/EditableLabelColumn";

const LabelColumn = ({ payment, value }) => {
  
  const updateLabel = async (newLabel) => {
    await labelPayment(newLabel, payment.paymentHash);
  }

  return <EditableLabelColumn 
    label={value} 
    updateLabel={updateLabel} 
    queryKey={"payments"} 
  />
  
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

const SimpleColumn = ({ value, className }) => {
  if (new Date(value).getTime() > 0) {
    value = new Date(value).toLocaleDateString("en-US");
  }

  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value}
    </td>
  );
};

const StatusColumn = ({ value, className }) => {
  return (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value === "pending" && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
          Pending
        </span>
      )}
      {value === "failed" && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
          Failed
        </span>
      )}
      {value === "succeeded" && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
          Paid
        </span>
      )}
    </td>
  );
};

const InvoiceColumn = ({ payment, value, className }) => {
  let [copied, setCopied] = useState(false);

  const copyClicked = () => {
    copy(payment.invoice);
    setCopied(true);
    setTimeout(() => {
      setCopied(false);
    }, 1000);
  };

  return !copied ? (
    <td
      onClick={copyClicked}
      className={`group cursor-pointer p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      {value}{" "}
      <span className="inline-block group-hover:hidden">
        &nbsp;&nbsp;&nbsp;&nbsp;
      </span>
      <ClipboardCopyIcon className="w-4 text-gray-500 hidden group-hover:inline-block" />
    </td>
  ) : (
    <td
      className={`p-3 md:px-6 md:py-4  whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
    >
      Copied!
    </td>
  );
};

const PaymentRow = ({ result, extraClass, attributes }) => {
  let columnKeyComponentMap = {
    amtMsat: AmountColumn,
    label: LabelColumn,
    displayInvoice: InvoiceColumn,
    status: StatusColumn,
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
            payment={result}
            value={value}
            className={className}
          />
        );
      })}
    </tr>
  );
};

const PaymentsList = ({ origin = "", status = "" }) => {
  const emptyTableHeadline = "No payments found";
  const emptyTableSubtext = "Try changing the search term";
  const searchBarPlaceholder = "Search";

  const attributes = [
    {
      key: "createdAt",
      label: "Created",
    },
    {
      key: "label",
      label: "Label",
    },
    {
      key: "displayInvoice",
      label: "Invoice",
    },
    {
      key: "displayPaymentHash",
      label: "Hash",
    },
    {
      key: "status",
      label: "Status",
    },
    {
      key: "amtMsat",
      label: "Amount (sats)",
    },
  ];

  const transformResults = (payments: Payment[]) => {
    return payments.map((payment) => {
      return {
        ...payment,
        createdAt: payment.createdAt * 1000,
        updatedAt: payment.updatedAt * 1000,
        displayPaymentHash: truncateMiddle(payment.paymentHash || "", 10),
        displayInvoice: truncateMiddle(payment.invoice || "", 10),
      };
    });
  };

  const refetchInterval = (data, query) => {
    const hasPendingPayment = data?.results.find(payment => {
      return payment.status === "pending"
    })
    return hasPendingPayment ? 1000 : false
  }

  const queryFunction = async ({ queryKey }) => {
    const [_key, { page, searchTerm, take }] = queryKey;
    const { payments, pagination } = await getPayments({
      page,
      searchTerm,
      take,
      origin,
      status,
    });

    return {
      results: transformResults(payments),
      hasMore: pagination.hasMore,
      total: pagination.total,
    };
  };

  return (
    <SearchableTable
      attributes={attributes}
      queryKey="payments"
      queryFunction={queryFunction}
      emptyTableHeadline={emptyTableHeadline}
      emptyTableSubtext={emptyTableSubtext}
      searchBarPlaceholder={searchBarPlaceholder}
      hasHeader
      itemsPerPage={5}
      RowComponent={PaymentRow}
      refetchInterval={refetchInterval}
    />
  );
};

export default PaymentsList;
