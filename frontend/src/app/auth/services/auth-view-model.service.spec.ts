import { TestBed } from '@angular/core/testing';

import { AuthViewModelService } from './auth-view-model.service';

describe('AuthViewModelService', () => {
  let service: AuthViewModelService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(AuthViewModelService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
