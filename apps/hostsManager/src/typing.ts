
export type Item = {
  id: string;
  name: string;
  on: boolean;
  system?: boolean;
  // Tree support
  type?: 'file' | 'folder'; // default file if undefined
  children?: Item[]; // only for folder
};

// Note: legacy group fields removed; existing persisted JSON with groupId/color will be ignored gracefully.

