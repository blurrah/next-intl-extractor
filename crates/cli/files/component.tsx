// @ts-nocheck
/* eslint-disable */
import { useTranslations } from "next-intl/client";
import { getTranslations } from "next-intl/server";

export function MyComponent() {
  const t = useTranslations("HelloFriends");

  return (
    <p>
      {t("hello")} {t("bye23456")}
    </p>
  );
}

export function SecondComponent() {
  const t = useTranslations("HelloFriends");
  const t2 = useTranslations("GoodbyeFriends.Test");

  return (
    <p>
      {t("goodbye")} {t("goodbye")} {t("hello")} {t("test")} {t2("hello")}{" "}
      {t.rich("willthiswork")}
    </p>
  );
}

export async function ServerComponent() {
  const t = await getTranslations({ namespace: "HelloFriends.Async" });

  return <p>{t("test")}</p>;
}
