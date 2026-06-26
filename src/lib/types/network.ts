export interface PortRule {
  label: string;
  port: number;
  protocol: string;
  ruleName: string;
  ruleExists: boolean;
  key: string;
}

export interface PortConflict {
  port: number;
  otherServer: string;
}

export interface NetworkStatus {
  firewallSupported: boolean;
  platform: string;
  ports: PortRule[];
  conflicts: PortConflict[];
}
