import {TestBed} from '@angular/core/testing';

import {RequestFailService} from './request-fail.service';

describe('RequestFailService', () => {
  let service: RequestFailService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(RequestFailService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
