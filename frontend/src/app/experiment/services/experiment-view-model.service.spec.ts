import { TestBed } from '@angular/core/testing';

import { ExperimentViewModelService } from './experiment-view-model.service';

describe('ExperimentViewModelService', () => {
  let service: ExperimentViewModelService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ExperimentViewModelService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
