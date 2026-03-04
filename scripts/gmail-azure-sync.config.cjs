module.exports = {
  apps: [
    {
      name: "gmail_azure_sync",
      script: "bun",

      args: ["src/lib/azure-sync-job-subscriber.ts"],

      cwd: "/home/dev/Local-Disk-F/vs-code/gmail-data-gateway-proj/gmail-data-gateway",
      
      exec_mode: "fork",
      instances: 1,

      autorestart: true,
      max_restarts: 3,
      min_uptime: "5000",
      restart_delay: 3000,
      exp_backoff_restart_delay: 100,
      kill_timeout: 3000,
      wait_ready: false,

      env: {
        NODE_ENV: "production"
      }
    }
  ]
};
