import useDebounce from "../../hooks/useDebounce";
import { useQuery, QueryKey } from "react-query";
import { Link } from "react-router-dom";
import format from "date-fns/format";
import { ReactNode, useEffect, useState } from "react";
import { FormattedMessage } from "react-intl";
import { SearchIcon } from "@heroicons/react/outline";
import Spinner from "src/components/Spinner";

export const SearchBar = ({ query = "", setQuery, placeholder, title }) => {
  return (
    <div className="p-4 bg-plum-100 text-light-plum flex items-center justify-between">
      {title && <span className="flex-grow-0 pr-4 font-bold">{title}</span>}
      <div
        className={`${
          title ? "max-w-[300px]" : ""
        } flex w-full items-center space-x-1 rounded-xl border border-transparent bg-plum-200 px-2 focus-within:border-orange focus-within:text-orange md:space-x-2 md:px-5`}
      >
        <span>
          <SearchIcon className="w-5" />
        </span>
        <input
          className="w-full border-none bg-transparent py-3 text-sm text-white outline-none placeholder:text-gray-300 focus:ring-0"
          type="text"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
          }}
          placeholder={placeholder}
        />
      </div>
    </div>
  );
};

export const SimpleRow = ({ result, extraClass, attributes }) => {
  return (
    <tr className={`${extraClass}`}>
      {attributes.map(({ key, className }) => {
        let value = result[key];
        if (typeof value === "object") {
          value = format(value, "MM/dd/YYY");
        }
        return (
          <td
            key={key}
            className={`p-3 md:px-6 md:py-4 whitespace-nowrap text-sm leading-5 font-medium text-light-plum ${className}`}
          >
            {value}
          </td>
        );
      })}
    </tr>
  );
};

export const ResultRow = ({
  result,
  index,
  attributes,
  striped,
  RowComponent,
}) => {
  const bgColor = striped ? "even:bg-gray-accent2 odd:bg-gray-accent3" : "";
  const linkClass = result.link ? "cursor-pointer hover:bg-plum-50" : "";

  if (result.link) {
    <Link to={result.link} key={index}>
      <RowComponent
        result={result}
        extraClass={`${linkClass} ${bgColor}`}
        attributes={attributes}
      />
    </Link>;
  } else {
    return (
      <RowComponent
        result={result}
        extraClass={bgColor}
        attributes={attributes}
      />
    );
  }
};

export const EmptyTable = ({ headline, subtext }) => {
  return (
    <div className="min-w-full bg-plum-100 p-5 text-center">
      <h1 className="text-lg">{headline}</h1>
      <p className="text-sm font-extralight">{subtext}</p>
    </div>
  );
};

export const SimpleTable = ({
  results,
  attributes,
  striped,
  hasHeader,
  headline,
  subtext,
  RowComponent,
}) => {
  if (results.length === 0) {
    return <EmptyTable headline={headline} subtext={subtext} />;
  }
  return (
    <table className="min-w-full">
      {hasHeader && (
        <thead>
          <tr>
            {attributes.map(({ key, label, className }) => {
              return (
                <th
                  key={key}
                  className={`p-3 md:px-6 md:py-4 bg-plum-100 text-left text-xs leading-4 font-bold text-plum-light uppercase tracking-wider ${className}`}
                >
                  {label}
                </th>
              );
            })}
          </tr>
        </thead>
      )}
      <tbody>
        {results.map((result, index) => (
          <ResultRow
            key={result.id}
            attributes={attributes}
            result={result}
            index={index}
            striped={striped}
            RowComponent={RowComponent}
          />
        ))}
      </tbody>
    </table>
  );
};

export const TableNavigation = ({
  start,
  end,
  total,
  canGoBack,
  canGoForward,
  goBack,
  goForward,
}) => {
  if (total === 0) {
    return null;
  }

  return (
    <nav className="bg-plum-100 px-4 py-3 flex items-center justify-between sm:px-6">
      <div className="hidden sm:block">
        <p className="text-sm leading-5 text-light-plum">
          <FormattedMessage
            id="searchable-table-pagination"
            defaultMessage="Showing <span>{startResult}</span> to <span>{endResult}</span> of <span>{totalResults}</span> results"
            description="Shows pagination information for tables"
            values={{
              startResult: start,
              endResult: end,
              totalResults: total,
              span: (chunks) => <span className="font-medium">{chunks}</span>,
            }}
          />
        </p>
      </div>
      <div className="flex-1 flex justify-between space-x-2 sm:justify-end">
        <button disabled={!canGoBack} onClick={goBack} className="btn-ghost">
          <FormattedMessage
            id="searchable-table-previous"
            defaultMessage="Previous"
            description="Button to go to the previous page of results"
          />
        </button>
        <button
          disabled={!canGoForward}
          onClick={goForward}
          className="btn-ghost"
        >
          <FormattedMessage
            id="searchable-table-next"
            defaultMessage="Next"
            description="Button to go to the next page of results"
          />
        </button>
      </div>
    </nav>
  );
};

interface TableAttribute {
  key: string;
  label: string;
}

export interface PaginationResults<T> {
  results: Array<T>;
  total: number;
  hasMore: boolean;
}

interface SearchableTableProps<T> {
  queryFunction: (a?: any) => Promise<PaginationResults<T>>;
  queryKey: QueryKey;
  attributes: TableAttribute[];
  emptyTableHeadline: string;
  emptyTableSubtext: string;
  itemsPerPage?: number;
  striped?: boolean;
  hasHeader?: boolean;
  className?: string;
  searchBarTitle?: string | null;
  searchBarPlaceholder: string;
  RowComponent?: ReactNode;
  refetchInterval?: number | false | ((data: any, query: any) => number | false);
}

const SimpleSearchableTable = <T extends object>({
  queryFunction,
  queryKey,
  attributes,
  emptyTableHeadline,
  emptyTableSubtext,
  itemsPerPage = 5,
  striped = false,
  hasHeader = false,
  className = "",
  searchBarTitle = null,
  searchBarPlaceholder,
  RowComponent = SimpleRow,
  refetchInterval = false
}: SearchableTableProps<T>) => {
  const [page, setPage] = useState(0);
  const [searchTerm, setSearchTerm] = useState("");
  const debouncedSearchTerm = useDebounce(searchTerm, 500);
  useEffect(() => {
    setSearchTerm(debouncedSearchTerm);
  }, [debouncedSearchTerm]);

  const skip = itemsPerPage * Number(page);
  const take = itemsPerPage;

  const { data, isLoading, isError } = useQuery(
    [queryKey, { page, searchTerm, skip, take }],
    queryFunction,
    { 
      keepPreviousData: true,
      refetchInterval
    }
  );

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-4">
        <Spinner />
      </div>
    );
  }

  if (isError) {
    return <div>error</div>;
  }

  const { results, total, hasMore } = data;
  const start = Math.min(total, page * itemsPerPage + 1);
  const end = Math.min(total, start + itemsPerPage - 1);

  const goToPage = (page) => {
    setPage(page);
  };

  const goToPreviousPage = () => {
    goToPage(page - 1);
  };
  const goToNextPage = () => {
    goToPage(page + 1);
  };

  return (
    <div className={`flex flex-col ${className}`}>
      <div className="">
        <div className="inline-block min-w-full py-2 align-middle">
          <div className="">
            <SearchBar
              query={searchTerm}
              setQuery={setSearchTerm}
              placeholder={searchBarPlaceholder}
              title={searchBarTitle}
            />

            <SimpleTable
              results={results}
              attributes={attributes}
              hasHeader={hasHeader}
              striped={striped}
              headline={emptyTableHeadline}
              subtext={emptyTableSubtext}
              RowComponent={RowComponent}
            />

            <TableNavigation
              start={start}
              end={end}
              total={total}
              canGoBack={page > 0}
              canGoForward={hasMore}
              goForward={goToNextPage}
              goBack={goToPreviousPage}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default SimpleSearchableTable;
