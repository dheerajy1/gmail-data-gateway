import { Layout } from "@/components/Layout";
import React from "react";

type Props = { title: string; bunVersion: string };

export function Home({ title, bunVersion }: Props): React.ReactElement {
  return (
    <Layout title={title}>
      <div className="max-w-(--width-container) w-full mx-auto p-4 md:p-8 leading-relaxed">
        <nav className="flex flex-wrap gap-4 mb-4 px-4 md:px-0">
          {[
            ["/", "Home"],
            ["/about", "About"],
            ["/openapi/json", "OpenAPI spec"],
            ["/openapi", "Scalar UI"],
            ["/healthz", "Health"],
          ].map(([href, label]) => (
            <a
              key={href}
              href={href}
              className="font-medium no-underline transition-colors duration-200 text-link hover:text-text-primary hover:underline"
            >
              {label}
            </a>
          ))}
        </nav>

        <section className="flex flex-col gap-4 bg-surface border border-border rounded-lg p-4 md:p-8 mt-8">
          <h1 className="text-3xl font-bold text-text-primary">
            Welcome to Gmail data gateway API, Elysia + Bun {bunVersion} on
            Vercel 🚀
          </h1>
          <p className="text-base leading-relaxed text-text">
            Complete foundational structure, type safety layer, deployment
            readiness, and basic login authentication for the Gmail data gateway
            api backend.
          </p>
          <img
            src="/logo.png"
            alt="Logo"
            className="w-(--width-logo) h-auto border border-border-thick rounded-md"
          />
        </section>
      </div>
    </Layout>
  );
}
