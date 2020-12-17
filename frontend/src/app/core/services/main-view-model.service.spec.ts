import {TestBed} from '@angular/core/testing';

import {MainViewModelService} from './main-view-model.service';

describe('MainViewmodelService', () => {
  let service: MainViewModelService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(MainViewModelService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
