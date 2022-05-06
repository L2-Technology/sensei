export const capitalizeWord = (s) => {
  return s.charAt(0).toUpperCase() + s.slice(1);
};

export const capitalizeAllWords = (sentence) => {
  return sentence.split(" ").map(capitalizeWord).join(" ");
};

export const truncateMiddle = (word, maxLength) => {
  let halfLength = maxLength / 2;
  return (
    word.substr(0, halfLength) +
    "..." +
    word.substring(word.length - halfLength)
  );
};
