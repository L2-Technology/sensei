import { SvgChevronLeft } from "src/components/icons";
import { Link } from "react-router-dom";

const BackLink = ({ text, href }) => {
  return (
    <Link to={href}>
      <div className="mb-4 flex ">
        <a className="cursor-pointer flex flex-grow-0 items-center text-sm text-gray-500 hover:text-gray-800">
          <SvgChevronLeft width="18" height="18" />
          <span className="">{text}</span>
        </a>
      </div>
    </Link>
  );
};

export default BackLink;
