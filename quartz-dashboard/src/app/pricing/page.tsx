'use client';

import Link from 'next/link';
import { useState } from 'react';
import { Database, Check, ChevronDown, ChevronUp } from 'lucide-react';

const plans = [
  {
    name: 'Free',
    price: '$0',
    period: '/mo',
    description: 'Perfect for prototyping and side projects.',
    features: [
      '1,000 vectors',
      '10,000 queries/mo',
      '384-dimension vectors',
      'HNSW search algorithm',
      'Community support',
    ],
    cta: 'Get Started Free',
    ctaLink: '/signup',
    disabled: false,
    highlighted: false,
  },
  {
    name: 'Pro',
    price: '$29',
    period: '/mo',
    description: 'For production apps that need more scale.',
    badge: 'Most Popular',
    features: [
      '100,000 vectors',
      '100,000 queries/mo',
      'Custom dimensions',
      'HNSW search algorithm',
      'Priority support',
    ],
    cta: 'Coming Soon',
    ctaLink: '#',
    disabled: true,
    highlighted: true,
  },
  {
    name: 'Scale',
    price: '$99',
    period: '/mo',
    description: 'For high-traffic apps with demanding SLAs.',
    features: [
      '1,000,000 vectors',
      'Unlimited queries',
      'Custom dimensions',
      'Dedicated support',
      'SLA guarantee',
    ],
    cta: 'Coming Soon',
    ctaLink: '#',
    disabled: true,
    highlighted: false,
  },
];

const faqs = [
  {
    question: 'What happens when I exceed my plan limits?',
    answer:
      'On the Free plan, API requests will return a 429 rate limit error once you exceed your monthly query or vector limits. You can upgrade your plan at any time to increase your limits.',
  },
  {
    question: 'Can I change plans later?',
    answer:
      'Yes! You can upgrade or downgrade your plan at any time. Changes take effect immediately and billing is prorated.',
  },
  {
    question: 'What vector dimensions are supported?',
    answer:
      'The Free plan supports 384-dimension vectors (compatible with models like all-MiniLM-L6-v2). Pro and Scale plans support custom dimensions up to 2048.',
  },
  {
    question: 'Is there a self-hosted option?',
    answer:
      'QuartzDB is designed to run on Cloudflare Workers. You can deploy your own instance using the open-source codebase — see the GitHub repository for instructions.',
  },
];

export default function PricingPage() {
  return (
    <div className="min-h-screen bg-gradient-to-b from-gray-900 via-gray-800 to-gray-900 text-white">
      {/* Header */}
      <header className="border-b border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <Link href="/" className="flex items-center gap-2">
              <Database className="h-8 w-8 text-cyan-400" />
              <span className="text-2xl font-bold">QuartzDB</span>
            </Link>
            <nav className="flex items-center gap-6">
              <Link href="/docs" className="text-gray-300 hover:text-white transition">
                Docs
              </Link>
              <Link href="/playground" className="text-gray-300 hover:text-white transition">
                Playground
              </Link>
              <Link
                href="/signup"
                className="bg-cyan-500 hover:bg-cyan-600 px-4 py-2 rounded-lg font-medium transition"
              >
                Get Started
              </Link>
            </nav>
          </div>
        </div>
      </header>

      {/* Hero */}
      <section className="py-16 px-4 text-center">
        <h1 className="text-4xl sm:text-5xl font-bold mb-4">Simple, Transparent Pricing</h1>
        <p className="text-gray-400 text-lg max-w-xl mx-auto">
          Start free, scale as you grow. No hidden fees.
        </p>
      </section>

      {/* Pricing Cards */}
      <section className="px-4 pb-20">
        <div className="max-w-5xl mx-auto grid md:grid-cols-3 gap-6">
          {plans.map((plan) => (
            <div
              key={plan.name}
              className={`relative bg-gray-800 rounded-xl p-6 flex flex-col ${
                plan.highlighted
                  ? 'border-2 border-cyan-500 shadow-lg shadow-cyan-500/10'
                  : 'border border-gray-700'
              }`}
            >
              {plan.badge && (
                <span className="absolute -top-3 left-1/2 -translate-x-1/2 bg-cyan-500 text-sm font-semibold px-3 py-0.5 rounded-full">
                  {plan.badge}
                </span>
              )}

              <h2 className="text-xl font-bold mb-1">{plan.name}</h2>
              <p className="text-gray-400 text-sm mb-4">{plan.description}</p>

              <div className="mb-6">
                <span className="text-4xl font-bold">{plan.price}</span>
                <span className="text-gray-400">{plan.period}</span>
              </div>

              <ul className="space-y-3 mb-8 flex-1">
                {plan.features.map((feature) => (
                  <li key={feature} className="flex items-center gap-2 text-sm text-gray-300">
                    <Check className="h-4 w-4 text-cyan-400 shrink-0" />
                    {feature}
                  </li>
                ))}
              </ul>

              {plan.disabled ? (
                <button
                  disabled
                  className="w-full bg-gray-700 text-gray-400 cursor-not-allowed px-4 py-3 rounded-lg font-medium"
                >
                  {plan.cta}
                </button>
              ) : (
                <Link
                  href={plan.ctaLink}
                  className={`w-full text-center px-4 py-3 rounded-lg font-medium transition ${
                    plan.highlighted
                      ? 'bg-cyan-500 hover:bg-cyan-600'
                      : 'bg-cyan-500 hover:bg-cyan-600'
                  }`}
                >
                  {plan.cta}
                </Link>
              )}
            </div>
          ))}
        </div>
      </section>

      {/* FAQ */}
      <section className="px-4 pb-20">
        <div className="max-w-3xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-10">Frequently Asked Questions</h2>
          <div className="space-y-3">
            {faqs.map((faq) => (
              <FAQItem key={faq.question} question={faq.question} answer={faq.answer} />
            ))}
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-gray-700 py-8 px-4">
        <div className="max-w-7xl mx-auto flex flex-col sm:flex-row justify-between items-center gap-4">
          <div className="flex items-center gap-2">
            <Database className="h-5 w-5 text-cyan-400" />
            <span className="font-semibold">QuartzDB</span>
          </div>
          <p className="text-gray-500 text-sm">Built with Cloudflare Workers &amp; Rust</p>
        </div>
      </footer>
    </div>
  );
}

function FAQItem({ question, answer }: { question: string; answer: string }) {
  const [open, setOpen] = useState(false);

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-xl">
      <button
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between px-5 py-4 text-left"
      >
        <span className="font-medium">{question}</span>
        {open ? (
          <ChevronUp className="h-5 w-5 text-gray-400 shrink-0" />
        ) : (
          <ChevronDown className="h-5 w-5 text-gray-400 shrink-0" />
        )}
      </button>
      {open && (
        <div className="px-5 pb-4 text-gray-400 text-sm">
          {answer}
        </div>
      )}
    </div>
  );
}
