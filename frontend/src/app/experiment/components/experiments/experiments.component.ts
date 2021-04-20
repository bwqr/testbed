import {Component, OnInit} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ExperimentViewModelService} from '../../services/experiment-view-model.service';
import {Experiment} from '../../models';
import {Pagination} from '../../../core/models';
import {PaginationService} from '../../../core/services/pagination.service';
import {ActivatedRoute} from '@angular/router';
import {switchMap} from 'rxjs/operators';
import {formats} from '../../../../defs';

@Component({
  selector: 'app-experiments',
  templateUrl: './experiments.component.html',
  styleUrls: ['./experiments.component.scss']
})
export class ExperimentsComponent extends MainComponent implements OnInit {

  experiments: Pagination<Experiment>;

  paginationRange: number[] = [];

  formats = formats;

  get isPageReady(): boolean {
    return !!this.experiments;
  }

  constructor(
    private viewModel: ExperimentViewModelService,
    private paginationService: PaginationService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.activatedRoute.queryParams.pipe(
        switchMap((params) => this.viewModel.experiments(this.paginationService.getPaginationFromParams(params)))
      ).subscribe(pagination => {
        this.experiments = pagination;
        this.paginationRange = Array(pagination.totalPages).fill(0).map((_, index) => index + 1);
      })
    );
  }

}
