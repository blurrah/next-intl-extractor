import { useTranslations } from "next-intl/client";

export function MyComponent() {
  const t = useTranslations("HelloFriends");

  return <p>{t("hello")}</p>;
}

export function SecondComponent() {
  const t = useTranslations("HelloFriends");
  const t2 = useTranslations("GoodbyeFriends");

  return (
    <p>
      {t("goodbye")} {t("goodbye")} {t("hello")} {t("test")} {t2("hello")}
    </p>
  );
}
