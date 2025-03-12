export interface MessageRepo {
  name: string;
  id: string;
  clone_url: string;
  owner: {
    login: string;
  };
}

export interface MessageData {
  repository: MessageRepo;
  installation: {
    id: number;
  };
  after: string;
}
