import { TestBed } from '@angular/core/testing';

import { UserViewModelService } from './user-view-model.service';

describe('UserViewModelService', () => {
  let service: UserViewModelService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(UserViewModelService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
