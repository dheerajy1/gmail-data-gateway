import React from "react";

type Props = {
  children: React.ReactNode;
  title: string;
};

export function Layout({ children, title }: Props) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>{title}</title>
        <link rel="stylesheet" href="/global.css" />
        <link rel="icon" href="/favicon.ico" />
      </head>
      <body className="min-h-screen antialiased">
        {children}
      </body>
    </html>
  );
}
