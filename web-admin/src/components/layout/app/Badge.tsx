export const Badge = ({ text, bgColor, textColor }) => {
  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded-md text-sm font-medium leading-5 bg-gray-100 text-gray-800 ${bgColor} ${textColor}`}
    >
      {text}
    </span>
  );
};

export const GreenBadge = ({ text }) => {
  return (
    <Badge text={text} bgColor="bg-green-100" textColor="text-green-800" />
  );
};

export const YellowBadge = ({ text }) => {
  return (
    <Badge text={text} bgColor="bg-yellow-100" textColor="text-yellow-800" />
  );
};
