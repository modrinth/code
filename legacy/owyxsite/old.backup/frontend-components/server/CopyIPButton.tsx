"use client";

import { useState, useCallback } from "react";

interface CopyIPButtonProps {
  ip: string;
}

export default function CopyIPButton({ ip }: CopyIPButtonProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(ip);
      } else {
        // Fallback
        const textArea = document.createElement("textarea");
        textArea.value = ip;
        textArea.style.cssText = "position:fixed;top:0;left:0;opacity:0";
        document.body.appendChild(textArea);
        textArea.focus();
        textArea.select();
        document.execCommand("copy");
        document.body.removeChild(textArea);
      }
      setCopied(true);
      setTimeout(() => setCopied(false), 2500);
    } catch {
      console.error("Failed to copy IP");
    }
  }, [ip]);

  return (
    <div className="relative">
      <button
        onClick={handleCopy}
        className="ip-copy-button inline-flex items-center bg-[#0a0a0a]/30 hover:bg-[#0a0a0a]/50 border border-[#FFAA00]/30 hover:border-[#FFAA00]/60 rounded-lg px-3 py-2 cursor-pointer transition-all duration-300 hover:scale-105"
        title="Нажмите, чтобы скопировать IP"
        id="copy-ip-btn"
      >
        <span className="font-mono text-[#FFAA00] hover:text-[#FFFF55] transition-colors">
          {ip}
        </span>
        <svg
          className="ml-2 w-4 h-4 text-[#FFAA00] hover:text-[#FFFF55] transition-colors"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          {copied ? (
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M5 13l4 4L19 7"
            />
          ) : (
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
            />
          )}
        </svg>
      </button>

      {/* Tooltip */}
      {copied && (
        <div className="absolute top-full left-1/2 transform -translate-x-1/2 mt-2 bg-green-900/95 text-green-300 border border-green-500/70 px-4 py-2 rounded-lg text-sm z-50 whitespace-nowrap shadow-lg animate-[dropdown-show_0.2s_ease-out]">
          <svg className="inline w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
          </svg>
          IP скопирован!
        </div>
      )}
    </div>
  );
}
