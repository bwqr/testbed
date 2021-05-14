import { TestBed } from '@angular/core/testing';

import { AdminViewModelService } from './admin-view-model.service';

describe('AdminViewModelService', () => {
  let service: AdminViewModelService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(AdminViewModelService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
