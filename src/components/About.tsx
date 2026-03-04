import { Layout } from "@/components/Layout";
import React from "react";

type AboutProps = {
  title: string;
  heading: string;
  description: string;
};

export function About({
  title,
  heading,
  description,
}: AboutProps): React.ReactElement {
  return (
    <Layout title={title}>
      <div className="mx-auto max-w-(--width-about) mt-10 px-4 md:px-0">
        <h1 className="text-2xl md:text-3xl font-bold mb-4 tracking-wider text-text-primary">
          {heading}
        </h1>
        <p className="leading-relaxed text-text bg-surface p-5 md:p-6 border border-border-thick rounded-lg">
          {description}
        </p>
      </div>
    </Layout>
  );
}
