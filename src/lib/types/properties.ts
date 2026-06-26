export interface PropertyEntry {
  key: string;
  value: string;
}

export interface PropertyUpdate {
  key: string;
  value: string;
}

/** Kind of input control the Settings UI renders for a property. */
export type PropertyControl = 'text' | 'number' | 'boolean' | 'select';

export interface PropertyFieldSpec {
  key: string;
  label: string;
  control: PropertyControl;
  options?: string[];
  help?: string;
}
