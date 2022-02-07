import React from "react";
import ReactDOM from "react-dom";
import "./index.css";
import App from "./App";
import { ModalProvider } from "./contexts/modal";
import { ConfirmProvider } from "./contexts/confirm";
import { ErrorProvider } from "./contexts/error";
import { NotificationProvider } from "./contexts/notification";
import reportWebVitals from "./reportWebVitals";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "react-query";
import { IntlProvider } from "react-intl";
import French from "./translations/fr.json";
import English from "./translations/en.json";

const locale = "en";

const localeToLangMap = {
  en: English,
  fr: French,
};

const getMessagesForLang = (locale: string): Record<string, string> => {
  return localeToLangMap[locale] as Record<string, string>;
};

const queryClient = new QueryClient();

ReactDOM.render(
  <React.StrictMode>
    <IntlProvider
      locale={locale}
      messages={getMessagesForLang(locale)}
      defaultLocale="en"
    >
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <NotificationProvider>
            <ErrorProvider>
              <ConfirmProvider>
                <ModalProvider>
                  <App />
                </ModalProvider>
              </ConfirmProvider>
            </ErrorProvider>
          </NotificationProvider>
        </BrowserRouter>
      </QueryClientProvider>
    </IntlProvider>
  </React.StrictMode>,
  document.getElementById("root")
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
